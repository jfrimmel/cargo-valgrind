use cargo_valgrind::{targets, Build, Cargo, Leak, Target, Valgrind};
use clap::{crate_authors, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::Colorize;
use std::path::{Path, PathBuf};

/// The Result type for this application.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The result of the valgrind run.
enum Report {
    /// The analyzed binary contains leaks.
    ContainsErrors,
    /// There was no error detected in the analyzed binary.
    NoErrorDetected,
}

/// Build the command line interface.
///
/// The CLI currently supports the distinction between debug and release builds
/// (selected via the `--release` flag) as well as the selection of the target
/// to execute. Currently binaries, examples and benches are supported.
fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("cargo valgrind")
        .version(crate_version!())
        .bin_name("cargo")
        .settings(&[AppSettings::GlobalVersion, AppSettings::SubcommandRequired])
        .subcommand(
            SubCommand::with_name("valgrind")
                .about("Cargo subcommand for running valgrind")
                .author(crate_authors!())
                .arg(
                    Arg::with_name("release")
                        .help("Build and run artifacts in release mode, with optimizations")
                        .long("release"),
                )
                .arg(
                    Arg::with_name("bin")
                        .help("Build and run the specified binary")
                        .long("bin")
                        .takes_value(true)
                        .value_name("NAME")
                        .conflicts_with_all(&["example", "bench", "test"]),
                )
                .arg(
                    Arg::with_name("example")
                        .help("Build and run the specified example")
                        .long("example")
                        .takes_value(true)
                        .value_name("NAME")
                        .conflicts_with_all(&["bin", "bench", "test"]),
                )
                .arg(
                    Arg::with_name("bench")
                        .help("Build and run the specified bench")
                        .long("bench")
                        .takes_value(true)
                        .value_name("NAME")
                        .conflicts_with_all(&["bin", "example", "test"]),
                )
                .arg(
                    Arg::with_name("test")
                        .help("Build and run the specified integration tests")
                        .long("test")
                        .takes_value(true)
                        .value_name("NAME")
                        .conflicts_with_all(&["bin", "example", "bench"]),
                )
                .arg(
                    Arg::with_name("manifest")
                        .help("Path to Cargo.toml")
                        .long("manifest-path")
                        .takes_value(true)
                        .value_name("PATH"),
                )
                .arg(
                    Arg::with_name("features")
                        .help("Space-separated list of features to activate")
                        .long("features")
                        .takes_value(true)
                        .value_name("FEATURES"),
                )
                .arg(
                    Arg::with_name("leak-check")
                        .help("Select, whether each leak or only a summary should be reported")
                        .long("leak-check")
                        .takes_value(true)
                        .value_name("KIND")
                        .possible_values(&["summary", "full"])
                        .default_value("summary"),
                )
                .arg(
                    Arg::with_name("leak-kinds")
                        .help(
                            "Select, which leak kinds to report (either a \
                             comma-separated list of `definite`, `indirect`, \
                             `possible` and `reachable` or `all`)",
                        )
                        .long("show-leak-kinds")
                        .takes_value(true)
                        .value_name("set")
                        .default_value("definite,possible")
                        .empty_values(false)
                        .validator(|s| {
                            if s == "all" {
                                Ok(())
                            } else {
                                s.split(',')
                                    .find(|&s| {
                                        s != "definite"
                                            && s != "indirect"
                                            && s != "possible"
                                            && s != "reachable"
                                    })
                                    .map_or(Ok(()), |s| Err(s.into()))
                            }
                        }),
                ),
        )
}

/// Query the build type (debug/release) from the the command line parameters.
fn build_type(parameters: &ArgMatches) -> Build {
    if parameters.is_present("release") {
        Build::Release
    } else {
        Build::Debug
    }
}

/// Query the path to the `Cargo.toml` from the the command line parameters.
///
/// This defaults to the current directory, if the `--manifest-path` parameter
/// is not given.
///
/// # Errors
/// This function fails, if the specified path is not valid.
fn manifest(parameters: &ArgMatches) -> Result<PathBuf> {
    let manifest = parameters.value_of("manifest").unwrap_or("Cargo.toml");
    let manifest = PathBuf::from(manifest).canonicalize()?;
    Ok(manifest)
}

/// Query the enabled features.
fn features<'a>(parameters: &'a ArgMatches) -> impl Iterator<Item = String> + 'a {
    parameters
        .value_of("features")
        .into_iter()
        .flat_map(|features| features.split(' '))
        .map(|feature| feature.into())
}

/// Query the specified `Target`, if any.
fn specified_target(parameters: &ArgMatches) -> Option<Target> {
    parameters
        .value_of("bin")
        .map(|path| Target::Binary(PathBuf::from(path)))
        .or_else(|| {
            parameters
                .value_of("example")
                .map(|path| Target::Example(PathBuf::from(path)))
        })
        .or_else(|| {
            parameters
                .value_of("bench")
                .map(|path| Target::Benchmark(PathBuf::from(path)))
        })
        .or_else(|| {
            parameters
                .value_of("test")
                .map(|path| Target::Test(PathBuf::from(path)))
        })
}

/// Search for the actual binary to analyze.
///
/// This function takes the output of `specified_target()`, as well as the list
/// of all possible targets returned by `targets()`. It searches, if the
/// requested binary exists. If no binary was specified and there is only one
/// target available, that target is used.
///
/// # Errors
/// This function returns an error, if there is no target specified and there
/// are multiple targets to choose from, or if the user specified a non-existing
/// target.
fn find_target(specified: Option<Target>, targets: &[Target]) -> Result<Target> {
    let target_type = |target: &Target| {
        if target.is_binary() {
            "bin"
        } else if target.is_example() {
            "example"
        } else if target.is_benchmark() {
            "bench"
        } else if target.is_test() {
            "test"
        } else {
            unreachable!();
        }
    };
    let target = match specified {
        Some(path) => path,
        None if targets.len() == 1 => targets[0].clone(),
        None if targets.is_empty() => return Err("No runnable target found.".into()),
        None => {
            let mut error = String::from("Multiple possible targets, please specify one of:\n");
            let targets: Vec<_> = targets
                .iter()
                .map(|target| {
                    let flag = target_type(target);
                    format!("--{} {}", flag, target.name())
                })
                .collect();

            error += &targets.join("\n");
            return Err(error.into());
        }
    };
    let target = targets
        .iter()
        .find(|&path| path.name().starts_with(target.name()))
        .cloned()
        .ok_or_else(|| {
            format!(
                "Could not find {} target `{}`",
                target_type(&target).replace("bin", "binary"),
                target.name()
            )
        })?;
    Ok(target)
}

/// Display a single `Leak` to the console.
fn display_error(leak: Leak) {
    println!(
        "{:>12} Leaked {}",
        "Error".red().bold(),
        bytesize::to_string(leak.leaked_bytes() as _, true)
    );
    let mut info = Some("Info".cyan().bold());
    for function in leak.back_trace() {
        println!("{:>12} at {}", info.take().unwrap_or_default(), function);
    }
}

/// Run the specified target inside of valgrind and print the output.
fn analyze_target(cli: &ArgMatches<'_>, target: &Target, manifest: &Path) -> Result<Report> {
    let crate_root = manifest.parent().ok_or("Invalid empty manifest path")?;
    let target_path = target
        .real_path()
        .strip_prefix(crate_root)
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    println!("{:>12} `{}`", "Analyzing".green().bold(), target_path);

    let mut valgrind = Valgrind::new();
    if let Some(kind) = cli.value_of("leak-check") {
        valgrind.set_leak_check(kind);
    }
    match cli.value_of("leak-kinds") {
        Some("all") => {
            valgrind.all_leak_kinds();
        }
        Some(kinds) => {
            valgrind.set_leak_kinds(&kinds.split(',').collect::<Vec<_>>());
        }
        _ => {}
    }
    let errors = valgrind.analyze(target.real_path())?;
    if errors.is_empty() {
        Ok(Report::NoErrorDetected)
    } else {
        let sum: usize = errors.iter().map(|leak| leak.leaked_bytes()).sum();
        errors.into_iter().for_each(display_error);
        println!(
            "{:>12} Leaked {} total",
            "Summary".red().bold(),
            bytesize::to_string(sum as _, true)
        );

        Ok(Report::ContainsErrors)
    }
}

fn run() -> Result<Report> {
    let cli = cli().get_matches();
    let cli = cli.subcommand_matches("valgrind").unwrap();
    let build = build_type(&cli);
    let target = specified_target(&cli);
    let manifest = manifest(&cli)?;
    let features = features(&cli);

    let targets = targets(&manifest, build)?;
    let target = find_target(target, &targets)?;
    Cargo::new()
        .manifest(&manifest)
        .build_target(target.clone())
        .build_type(build)
        .features(features)
        .build()?;
    analyze_target(&cli, &target, &manifest)
}

fn main() {
    match run() {
        Ok(Report::NoErrorDetected) => {}
        Ok(Report::ContainsErrors) => std::process::exit(1),
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}
