use cargo_valgrind::{targets, build_target, valgrind, Build, Target};
use clap::{crate_authors, crate_name, crate_version, App, Arg};
use colored::Colorize;
use std::path::PathBuf;

/// Build the command line interface.
///
/// The CLI currently supports the distinction between debug and release builds
/// (selected via the `--release` flag) as well as the selection of the target
/// to execute. Currently binaries, examples and benches are supported.
fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .about("Cargo subcommand for running valgrind")
        .author(crate_authors!())
        .version(crate_version!())
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
                .conflicts_with_all(&["example", "bench"]),
        )
        .arg(
            Arg::with_name("example")
                .help("Build and run the specified example")
                .long("example")
                .takes_value(true)
                .value_name("NAME")
                .conflicts_with_all(&["bin", "bench"]),
        )
        .arg(
            Arg::with_name("bench")
                .help("Build and run the specified bench")
                .long("bench")
                .takes_value(true)
                .value_name("NAME")
                .conflicts_with_all(&["bin", "example"]),
        )
        .arg(
            Arg::with_name("manifest")
                .help("Path to Cargo.toml")
                .long("manifest-path")
                .takes_value(true)
                .value_name("PATH"),
        )
}

fn run() -> Result<bool, Box<dyn std::error::Error>> {
    let cli = cli().get_matches();
    let build = if cli.is_present("release") {
        Build::Release
    } else {
        Build::Debug
    };
    let binary = cli
        .value_of("bin")
        .map(|path| Target::Binary(PathBuf::from(path)))
        .or(cli.value_of("example")
        .map(|path| Target::Example(PathBuf::from(path))))
        .or(cli.value_of("bench")
        .map(|path| Target::Benchmark(PathBuf::from(path))));
    let manifest = cli.value_of("manifest").unwrap_or("Cargo.toml".into());
    let manifest = PathBuf::from(manifest).canonicalize()?;

    let binaries = targets(&manifest, build)?;
    let binary = match binary {
        Some(path) => path,
        None if binaries.len() == 1 => binaries[0].clone(),
        None => Err("Multiple possible targets, please specify more precise")?,
    };
    let binary = binaries
        .into_iter()
        .find(|path| path.name() == binary.name())
        .ok_or("Could not find selected binary")?;
    let crate_root = manifest.parent().unwrap();
    let target_path = binary
        .path()
        .strip_prefix(crate_root)
        .map(|path| path.display().to_string())
        .unwrap_or_default();

    build_target(&manifest, build, binary.clone())?;
    println!("{:>12} `{}`", "Analyzing".green().bold(), target_path);

    let report = valgrind(binary.path())?;
    if report.len() >= 1 {
        for error in report {
            println!(
                "{:>12} Leaked {} bytes",
                "Error".red().bold(),
                error.leaked_bytes()
            );
            let mut info = Some("Info".cyan().bold());
            for function in error.back_trace() {
                println!("{:>12} at {}", info.take().unwrap_or_default(), function);
            }
        }
        Ok(false)
    } else {
        Ok(true)
    }
}

fn main() {
    match run() {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}
