//! This module provides utility functions around valgrind.

pub mod xml;

use std::net::{SocketAddr, TcpListener};
use std::process::Command;
use std::{env, fmt, io::Read};
use std::{ffi::OsStr, process::Stdio};
use strong_xml::{XmlError, XmlRead};

/// Error type for valgrind-execution-related failures.
#[derive(Debug)]
pub enum Error {
    /// The `valgrind` binary is not installed or not executable.
    ///
    /// This is an user error.
    ValgrindNotInstalled,
    /// Something around the socket creation did fail.
    SocketConnection,
    /// The sub-process could not be waited on.
    ProcessFailed,
    /// Valgrind execution did fail.
    ///
    /// The error output of valgrind is captured.
    ValgrindFailure(String),
    /// The valgrind output was malformed or otherwise unexpected.
    ///
    /// This variant contains the inner deserialization error and the output of
    /// valgrind.
    MalformedOutput(XmlError, Vec<u8>),
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ValgrindNotInstalled => write!(f, "valgrind executable not found"),
            Self::SocketConnection => write!(f, "local TCP I/O error"),
            Self::ProcessFailed => write!(f, "cannot start valgrind process"),
            Self::ValgrindFailure(s) => write!(f, "invalid valgrind usage: {}", s),
            Self::MalformedOutput(e, _) => write!(f, "unexpected valgrind output: {}", e),
        }
    }
}

/// Execute a certain command inside of valgrind and collect the [`Output`].
///
/// [`Output`]: xml::Output
pub fn execute<S, I>(command: I) -> Result<xml::Output, Error>
where
    S: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
{
    // open a TCP socket on localhost, port selected by OS
    let address: SocketAddr = ([127, 0, 0, 1], 0).into();
    let listener = TcpListener::bind(address).map_err(|_| Error::SocketConnection)?;
    let address = listener.local_addr().map_err(|_| Error::SocketConnection)?;

    let mut valgrind = Command::new("valgrind");

    // additional options to pass to valgrind?
    if let Ok(additional_args) = env::var("VALGRINDFLAGS") {
        valgrind.args(additional_args.split(" "));
    }

    let cargo = valgrind
        .arg("--xml=yes")
        .arg(format!("--xml-socket={}:{}", address.ip(), address.port()))
        .args(command)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| Error::ValgrindNotInstalled)?;

    // spawn a new thread, that receives the XML and parses it. This has to be
    // a separate execution unit (a thread is currently used, but an `async`
    // task would be suitable as well), as the `accept()` call blocks until the
    // valgrind binary writes something to the TCP connection. This is normally
    // fine, but if we consider errors, e.g. wrong command line flags, valgrind
    // won't write anything to the connection, so the program will hang forever.
    // The thread can simply be thrown away, if valgrind fails.
    let xml = std::thread::spawn(move || {
        // collect the output of valgrind
        let (mut listener, _) = listener.accept().map_err(|_| Error::SocketConnection)?;
        let mut output = Vec::new();
        listener
            .read_to_end(&mut output)
            .map_err(|_| Error::SocketConnection)?;
        let output_str = match std::str::from_utf8(&output) {
            Ok(s) => s,
            Err(e) => return Err(Error::MalformedOutput(XmlError::Utf8(e), output))?,
        };
        let xml: xml::Output =
            xml::Output::from_str(output_str).map_err(|e| Error::MalformedOutput(e, output))?;
        Ok(xml)
    });

    let output = cargo.wait_with_output().map_err(|_| Error::ProcessFailed)?;
    if output.status.success() {
        let xml = xml.join().expect("Reader-thread panicked")?;
        Ok(xml)
    } else {
        // this does not really terminalte the thread, but detaches it. Despite
        // that, the thread will be killed, if the main thread exits.
        drop(xml);
        Err(Error::ValgrindFailure(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    // TODO: use drop guard, that waits on child in order to prevent printing to stdout of the child
}
