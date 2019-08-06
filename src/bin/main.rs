use cargo_valgrind::{binaries, build_target, valgrind, Build, Target};
use clap::{crate_authors, crate_name, crate_version, App, Arg};
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

fn main() {
    let cli = cli().get_matches();
    let build = if cli.is_present("release") {
        Build::Release
    } else {
        Build::Debug
    };
    let binary = cli
        .value_of("bin")
        .or(cli.value_of("example"))
        .or(cli.value_of("bench"));
    let manifest = cli.value_of("manifest").unwrap_or("Cargo.toml".into());

    let binaries = binaries(&manifest, build).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let binary = binary
        .map(PathBuf::from)
        .or_else(|| {
            if binaries.len() == 1 {
                binaries
                    .get(0)
                    .map(|path| PathBuf::from(path))
                    .map(|path| path.file_name().unwrap().into())
            } else {
                eprintln!("Multiple possible targets, please specify more precise");
                std::process::exit(1);
            }
        })
        .and_then(|binary| {
            binaries.into_iter().find(|path| {
                path.file_name()
                    .map(|target| target == binary)
                    .unwrap_or(false)
            })
        })
        .unwrap_or_else(|| {
            eprintln!("error: could not find specified executable");
            std::process::exit(1);
        });

    let manifest = PathBuf::from(manifest).canonicalize().unwrap();
    let crate_root = manifest.parent().unwrap();

    build_target(
        &manifest,
        build,
        Target::Binary(binary.file_name().unwrap().into()),
    )
    .unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    println!(
        "{:>12} `{}`",
        "Analyzing",
        binary
            .strip_prefix(crate_root)
            .map(|path| path.display().to_string())
            .unwrap_or_default()
    );
    let report = valgrind(&binary).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    if report.len() >= 1 {
        for error in report {
            println!("{:>12} Leaked {} bytes", "Error", error.leaked_bytes());
            let mut info = Some("Info");
            for function in error.back_trace() {
                println!("{:>12} at {}", info.take().unwrap_or_default(), function);
            }
        }
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}
