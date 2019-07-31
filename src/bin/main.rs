use clap::{crate_authors, crate_name, crate_version, App, Arg};

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
        "release"
    } else {
        "debug"
    };
    let binary = cli
        .value_of("bin")
        .or(cli.value_of("example"))
        .or(cli.value_of("bench"));
    let manifest = cli.value_of("manifest").unwrap_or("Cargo.toml".into());

    println!("Manifest: {}", manifest);
    println!("Build type: {}", build);
    println!("Test artifact: {:?}", binary);
}
