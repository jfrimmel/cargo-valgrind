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
}

fn main() {
    println!("Hello, world!");
}
