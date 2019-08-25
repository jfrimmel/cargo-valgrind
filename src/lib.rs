//! The core library of the `cargo-valgrind` command.
mod metadata;
#[cfg(test)]
mod tests;
mod valgrind_xml;

use std::{
    fmt::{self, Display, Formatter},
    io::{Error, ErrorKind},
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    process::{Command, Output},
};
pub use valgrind_xml::Kind;

/// The possible build types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Build {
    /// This is a debug build.
    Debug,
    /// This is a release build.
    Release,
}
impl Default for Build {
    fn default() -> Self {
        Build::Debug
    }
}
impl AsRef<Path> for Build {
    fn as_ref(&self) -> &Path {
        match self {
            Build::Debug => Path::new("debug"),
            Build::Release => Path::new("release"),
        }
    }
}

/// The possible targets to build and run within valgrind.
#[derive(Debug, Clone, Eq)]
pub enum Target {
    /// A normal binary with the given name.
    Binary(PathBuf),
    /// An example with the given name.
    Example(PathBuf),
    /// A benchmark with the given name.
    Benchmark(PathBuf),
    /// A test with the given name.
    Test(PathBuf),
}
impl Target {
    /// Query the path to the target binary.
    pub fn path(&self) -> &Path {
        match self {
            Target::Binary(path)
            | Target::Example(path)
            | Target::Benchmark(path)
            | Target::Test(path) => path.as_path(),
        }
    }

    /// Query the name of the target binary.
    ///
    /// # Panics
    /// This method panics, if either the path has no file name, i.e. it is
    /// empty or the file name contains invalid UTF-8.
    pub fn name(&self) -> &str {
        self.path()
            .file_name()
            .expect("binary has no name")
            .to_str()
            .expect("binary name contained invalid UTF-8")
    }

    /// Query, if the target is an ordinary binary.
    pub fn is_binary(&self) -> bool {
        match self {
            Target::Binary(_) => true,
            _ => false,
        }
    }

    /// Query, if the target is an example binary.
    pub fn is_example(&self) -> bool {
        match self {
            Target::Example(_) => true,
            _ => false,
        }
    }

    /// Query, if the target is a benchmark binary.
    pub fn is_benchmark(&self) -> bool {
        match self {
            Target::Benchmark(_) => true,
            _ => false,
        }
    }

    /// Query, if the target is a test binary.
    pub fn is_test(&self) -> bool {
        match self {
            Target::Test(_) => true,
            _ => false,
        }
    }
}
impl std::cmp::PartialEq for Target {
    fn eq(&self, other: &Target) -> bool {
        self.name() == other.name()
            && match (self, other) {
                (Target::Binary(_), Target::Binary(_))
                | (Target::Example(_), Target::Example(_))
                | (Target::Benchmark(_), Target::Benchmark(_))
                | (Target::Test(_), Target::Test(_)) => true,
                _ => false,
            }
    }
}
impl std::hash::Hash for Target {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        std::mem::discriminant(self).hash(state);
    }
}

/// Invoke `cargo` and build the specified target.
///
/// The crate is specified by the path to the `Cargo.toml` using the `manifest`
/// parameter. The kind of build (debug or release) is selected via the `build`
/// parameter. The binary to build is specified via the `target` parameter.
///
/// # Errors
/// This function returns an error, if the `cargo command returned an error`.
#[deprecated(since = "1.1.0", note = "Use the more flexible `Cargo` type instead")]
pub fn build_target<P: AsRef<Path>>(
    manifest: P,
    build: Build,
    target: Target,
) -> Result<(), Error> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if let Build::Release = build {
        cmd.arg("--release");
    }
    cmd.arg("--manifest-path");
    cmd.arg(manifest.as_ref());
    match target {
        Target::Binary(_) => cmd.arg("--bin"),
        Target::Example(_) => cmd.arg("--example"),
        Target::Benchmark(_) => cmd.arg("--bench"),
        Target::Test(_) => cmd.arg("--test"),
    };
    cmd.arg(target.name());
    cmd.spawn()?.wait_with_output().and_then(|output| {
        if output.status.success() {
            Ok(())
        } else {
            Err(cargo_error(output))
        }
    })
}

/// A `cargo` build command.
///
/// This type acts as a sentinel for a `cargo build` process. It allows the
/// configuration via the `new()` function in a builder pattern style.
pub struct Cargo {
    /// The path to the manifest (`Cargo.toml`).
    manifest: PathBuf,
    /// The build type (debug or release).
    build: Build,
    /// The target binary.
    target: Target,
    /// Enabled features.
    features: Vec<String>,
}
impl Cargo {
    /// Start configuring the cargo command that will build the selected target.
    pub fn new() -> cargo_config::Manifest {
        cargo_config::Manifest::new()
    }

    /// Build the selected target with the previously specified configuration.
    ///
    /// This invokes `cargo` and builds the specified target. The crate is
    /// specified by the path to the `Cargo.toml` using the `manifest` field.
    /// The kind of build (debug or release) is selected via the `build` field.
    /// The binary to build is specified via the `target` field.
    ///
    /// # Errors
    /// This function returns an error, if the `cargo` command returned an
    /// error, e.g. because the manifest could not be found or a target is not
    /// available.
    pub fn build(&self) -> Result<(), Error> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        if let Build::Release = self.build {
            cmd.arg("--release");
        }
        cmd.arg("--manifest-path");
        cmd.arg(self.manifest.as_path());
        match self.target {
            Target::Binary(_) => cmd.arg("--bin"),
            Target::Example(_) => cmd.arg("--example"),
            Target::Benchmark(_) => cmd.arg("--bench"),
            Target::Test(_) => cmd.arg("--test"),
        };
        cmd.arg(self.target.name());
        if !self.features.is_empty() {
            cmd.arg("--features");
            cmd.arg(self.features.join(" "));
        }
        cmd.spawn()?.wait_with_output().and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(cargo_error(output))
            }
        })
    }

    /// Build the target with a specific feature enabled.
    ///
    /// This function can be called multiple times to build with multiple
    /// features enabled.
    pub fn feature<S: Into<String>>(self, feature: S) -> Self {
        self.features(Some(feature))
    }

    /// Build the target with specific features enabled.
    ///
    /// This function can be called multiple times and mixed with the
    /// `feature()` method.
    pub fn features<S, I>(mut self, features: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let features = features.into_iter().map(|feature| feature.into());
        self.features.extend(features);
        self
    }
}

pub mod cargo_config {
    //! A module containing the builder pattern types for configuring a `Cargo`.
    use super::{Build, Cargo, Target};
    use std::path::{Path, PathBuf};

    /// A `Cargo` instance while configuring its manifest path.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Manifest(());
    impl Manifest {
        pub(super) fn new() -> Self {
            Self(())
        }

        /// Specify the path to the `Cargo.toml` to use.
        pub fn manifest<P: AsRef<Path>>(self, manifest: P) -> BuildTarget {
            let manifest = manifest.as_ref().into();
            BuildTarget { manifest }
        }
    }

    /// A `Cargo` instance while configuring the target binary.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct BuildTarget {
        manifest: PathBuf,
    }
    impl BuildTarget {
        /// Select the build target.
        pub fn build_target(self, target: Target) -> BuildType {
            BuildType {
                manifest: self.manifest,
                target,
            }
        }
    }

    /// A `Cargo` instance while configuring the target build type.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct BuildType {
        manifest: PathBuf,
        target: Target,
    }
    impl BuildType {
        /// Select the build type.
        pub fn build_type(self, build: Build) -> Cargo {
            Cargo {
                manifest: self.manifest,
                target: self.target,
                build,
                features: vec![],
            }
        }

        /// Make a debug build.
        pub fn debug_build(self) -> Cargo {
            self.build_type(Build::Debug)
        }

        /// Make a release build.
        pub fn release_build(self) -> Cargo {
            self.build_type(Build::Release)
        }
    }
}

/// Run the program denoted by `path` in valgrind.
///
/// This function runs the program in valgrind, parses its XML output, collects
/// the leak information and returns the list of leaks. If this list is empty,
/// the program has no detected leaks.
///
/// # Errors
/// This function returns an error, if valgrind could not be executed
/// successfully, its output could not be parsed correctly or any other process
/// related error occurs.
#[deprecated(
    since = "1.2.0",
    note = "Use the more flexible `Valgrind` type instead"
)]
pub fn valgrind<P: AsRef<Path>>(path: P) -> Result<Vec<Leak>, Error> {
    Valgrind::new().analyze(path)
}

/// A `valgrind` command.
///
/// This type acts as a sentinel for a `valgrind` process. It allows the
/// configuration via the `new()` function in a builder pattern style.
#[derive(Debug)]
pub struct Valgrind {
    /// The valgrind command to execute.
    valgrind: Command,
}
impl Valgrind {
    /// Start configuring the valgrind command that will analyze the target.
    pub fn new() -> Self {
        Valgrind {
            valgrind: Command::new("valgrind"),
        }
    }

    /// Analyze the specified binary with the selected valgrind configuration.
    ///
    /// This function runs the program in valgrind, parses its XML output,
    /// collects the leak information and returns the list of leaks. If this
    /// list is empty, the program has no detected leaks.
    ///
    /// # Errors
    /// This function returns an error, if valgrind could not be executed
    /// successfully, its output could not be parsed correctly or any other
    /// process related error occurs.
    pub fn analyze<P: AsRef<Path>>(mut self, path: P) -> Result<Vec<Leak>, Error> {
        Ok(self
            .run_in_valgrind(path)?
            .errors
            .unwrap_or_default()
            .into_iter()
            .map(|error| Leak {
                bytes: error.resources.bytes,
                kind: error.kind,
                stack_trace: error
                    .stack_trace
                    .frames
                    .into_iter()
                    .map(|frame| Function {
                        name: frame.function,
                        file: frame.file,
                        line: frame.line,
                    })
                    .collect(),
            })
            .collect())
    }

    /// Specify, whether all leaks or only a summary should be reported.
    ///
    /// Possible values are `"summary"` and `"full"`. Other values will cause
    /// the valgrind command to fail.
    pub fn set_leak_check(&mut self, kind: &str) -> &mut Self {
        self.valgrind.arg(format!("--leak-check={}", kind));
        self
    }

    /// Run a binary inside `valgrind` and collect the report.
    ///
    /// This function launches a valgrind process, that does full leak checks
    /// and reports all leak kinds in the XML format. The XML output is sent to
    /// a local socket and then parsed into the `valgrind_xml::Output`
    /// structure.
    ///
    /// # Errors
    /// This function fails, if either the valgrind command couldn't be spawned
    /// or executed successfully, the socket creation or read operation fails or
    /// the received XML could not be parsed correctly.
    fn run_in_valgrind<P: AsRef<Path>>(&mut self, path: P) -> Result<valgrind_xml::Output, Error> {
        // port selected by OS
        let address: SocketAddr = ([127, 0, 0, 1], 0).into();
        let listener = TcpListener::bind(address)?;
        let address = listener.local_addr()?;
        let mut valgrind = self
            .valgrind
            .arg("--xml=yes")
            .arg(format!("--xml-socket={}:{}", address.ip(), address.port()))
            .arg(path.as_ref())
            .spawn()?;
        let (listener, _socket) = listener.accept()?;

        if valgrind.wait()?.success() {
            serde_xml_rs::from_reader(listener)
                .map_err(|e| Error::new(ErrorKind::Other, format!("Could not parse XML: {}", e)))
        } else {
            Err(Error::new(ErrorKind::Other, "valgrind command failed"))
        }
    }
}

/// A single memory leak.
///
/// This type holds the information about the leak, which includes the number of
/// leaked bytes, the kind of leak and the call trace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Leak {
    /// The number of bytes leaked.
    bytes: usize,
    /// The kind of leak.
    kind: Kind,
    /// The calling function, that caused the leak.
    stack_trace: Vec<Function>,
}
impl Leak {
    /// Query the amount of leaked bytes.
    pub fn leaked_bytes(&self) -> usize {
        self.bytes
    }

    /// Query the kind of the leak.
    pub fn leak_kind(&self) -> Kind {
        self.kind
    }

    /// Query the call trace, i.e. the functions, that lead to the leak.
    ///
    /// The functions are in the "most recent call first" order.
    pub fn back_trace(&self) -> &Vec<Function> {
        &self.stack_trace
    }
}

/// A single function in the call trace.
///
/// A function is denoted by its name and its file including the line. Note,
/// that all of this information may be absent. Valgrind can only output those
/// information, if the underlying objects have debug information associated
/// with them.
///
/// A `Function` implemented the `Display` trait, in which the available
/// information are printed in the following scheme `name (file:line)`, where
/// unavailable information are omitted. If the function name is not available,
/// it is replaced with `"unknown"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    /// The name of the function, if there is any debug information.
    name: Option<String>,
    /// The file name of the function, if there is any debug information.
    file: Option<String>,
    /// The line in the function, if there is any debug information.
    line: Option<usize>,
}
impl Function {
    /// Query the name of the function.
    ///
    /// This information may not be present, e.g. if the corresponding object is
    /// built without debug info.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|name| name.as_str())
    }

    /// Query the name of the file in which the called function was defined.
    ///
    /// This information may not be present, e.g. if the corresponding object is
    /// built without debug info.
    pub fn file(&self) -> Option<&str> {
        self.file.as_ref().map(|name| name.as_str())
    }

    /// Query the line of the function of the function call.
    ///
    /// This information may not be present, e.g. if the corresponding object is
    /// built without debug info.
    pub fn line(&self) -> Option<usize> {
        self.line
    }
}
impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.name().as_ref().unwrap_or(&"unknown"))?;
        if let Some(file) = &self.file() {
            f.write_str(" (")?;
            f.write_str(file)?;
            if let Some(line) = self.line() {
                write!(f, ":{}", line)?;
            }
            f.write_str(")")?;
        }
        Ok(())
    }
}

/// Query all binaries of the crate denoted by the given `Cargo.toml`.
///
/// This function returns the paths to each executable in the given crate. Those
/// are all the examples, benches as the actual crate binaries. This is based on
/// the crate metadata obtained by [`metadata()`](fn.metadata.html).
///
/// Only binaries of the specified manifest are returned. This means, that other
/// crates in the same workspace may have binaries, but they are ignored.
///
/// Note, that plain tests and `custom-build` kinds currently are not supported.
///
/// # Errors
/// This function fails for the same reasons as the `metadata()` function.
///
/// # Panics
/// This function currently panics, if a test or custom build binary is
/// encountered.
#[deprecated(note = "use targets() instead, as it provides more information")]
pub fn binaries<P: AsRef<Path>>(path: P, build: Build) -> Result<Vec<PathBuf>, Error> {
    let package = metadata(&path)?;
    let path = path.as_ref().canonicalize()?;
    Ok(binaries_from(package, path, build)?
        .into_iter()
        .map(|target| match target {
            Target::Binary(path)
            | Target::Example(path)
            | Target::Benchmark(path)
            | Target::Test(path) => path,
        })
        .collect())
}

/// Query all targets of the crate denoted by the given `Cargo.toml`.
///
/// This function returns the paths to and type of each executable in the given
/// crate. Those are all the examples, benches as the actual crate binaries.
/// This is based on the crate metadata obtained by
/// [`metadata()`](fn.metadata.html).
///
/// Only binaries of the specified manifest are returned. This means, that other
/// crates in the same workspace may have binaries, but they are ignored.
///
/// Note, that plain tests and `custom-build` kinds currently are not supported.
///
/// # Errors
/// This function fails for the same reasons as the `metadata()` function.
///
/// # Panics
/// This function currently panics, if a test or custom build binary is
/// encountered.
pub fn targets<P: AsRef<Path>>(path: P, build: Build) -> Result<Vec<Target>, Error> {
    let package = metadata(&path)?;
    let path = path.as_ref().canonicalize()?;
    binaries_from(package, path, build)
}

/// Query all binaries of given metadata.
///
/// See [`binaries()`](fn.binaries.html) for details.
///
/// This is the real implementation of the `binaries()` function. It was added
/// in order to be able to test this function without actual `Cargo.toml`s and
/// by giving prepared metadata.
///
/// Note, that the path denoted by `requested` has to be canonicalized before.
fn binaries_from<P: AsRef<Path>>(
    package: metadata::Metadata,
    requested: P,
    build: Build,
) -> Result<Vec<Target>, Error> {
    let target_dir = package.target_directory.join(build);
    Ok(package
        .packages
        .into_iter()
        .filter(|package| package.manifest_path == requested.as_ref())
        .flat_map(|package| {
            package
                .targets
                .into_iter()
                .filter(|target| target.crate_types.contains(&metadata::CrateType::Binary))
                .map(|target| {
                    let path = target_dir
                        .join(match target.kind[0] {
                            metadata::Kind::Binary => "",
                            metadata::Kind::Example => "examples",
                            metadata::Kind::Bench => "benches",
                            metadata::Kind::Test | metadata::Kind::CustomBuild => unimplemented!(),
                            metadata::Kind::Library
                            | metadata::Kind::ProcMacro
                            | metadata::Kind::DyLib
                            | metadata::Kind::CDyLib
                            | metadata::Kind::StaticLib
                            | metadata::Kind::RLib => unreachable!("Non-binaries are filtered out"),
                        })
                        .join(target.name);
                    match target.kind[0] {
                        metadata::Kind::Binary => Target::Binary(path),
                        metadata::Kind::Example => Target::Example(path),
                        metadata::Kind::Bench => Target::Benchmark(path),
                        metadata::Kind::Test => Target::Test(path),
                        _ => unreachable!("Unsupported target type"),
                    }
                })
        })
        .collect())
}

/// Query the crate metadata of the given `Cargo.toml`.
///
/// This collects the metadata of the crate denoted by the `path` using the
/// [`cargo_metadata()`](fn.cargo_metadata.html) function. Its output is then
/// parsed into the `Metadata` structure.
///
/// # Errors
/// This function either fails because of an error of the `cargo_metadata()`
/// function or due to an invalid output by it, that could not successfully be
/// parsed.
fn metadata<P: AsRef<Path>>(path: P) -> Result<metadata::Metadata, Error> {
    let metadata = cargo_metadata(path)?;
    serde_json::from_str(&metadata)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Invalid metadata: {}", e)))
}

/// Run the `cargo metadata` command and collect its output.
///
/// The `path` has to point to the `Cargo.toml` of which the metadata should be
/// collected. Metadata of the dependencies is omitted on purpose. The output is
/// then converted into a `String`.
///
/// # Errors
/// This function can fail either because the `cargo metadata` command could not
/// be spawned, the command failed (i.e. it was executed but returned a non-zero
/// exit code) or the string printed to stdout was not valid UTF-8.
fn cargo_metadata<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(path.as_ref())
        .output()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "Non-UTF-8 string"))
    } else {
        Err(cargo_error(output))
    }
}

/// Build an `io::Error` from the stderr text outputted by `cargo`.
fn cargo_error(output: Output) -> Error {
    let msg = String::from_utf8_lossy(&output.stderr);
    let msg = msg.trim_start_matches("error: ").trim_end();
    if msg.is_empty() {
        Error::new(ErrorKind::Other, "cargo command failed")
    } else {
        Error::new(ErrorKind::Other, format!("cargo command failed: {}", msg))
    }
}
