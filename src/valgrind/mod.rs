//! This module provides utility functions around valgrind.

pub mod xml;

use std::ffi::OsStr;
use std::net::{SocketAddr, TcpListener};
use std::process::Command;

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
    ValgrindFailure,
    /// The valgrind output was malformed or otherwise unexpected.
    MalformedOutput(serde_xml_rs::Error),
}
// TODO: impl std::error::Error for Error
// TODO: impl fmt::Display for Error

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

    let mut cargo = Command::new("valgrind")
        .arg("--xml=yes")
        .arg(format!("--xml-socket={}:{}", address.ip(), address.port()))
        .args(command)
        .spawn()
        .map_err(|_| Error::ValgrindNotInstalled)?;

    // collect the output of valgrind
    let (listener, _) = listener.accept().map_err(|_| Error::SocketConnection)?;
    let xml: xml::Output = serde_xml_rs::from_reader(listener).map_err(Error::MalformedOutput)?;

    match cargo.wait() {
        Ok(result) if result.success() => Ok(xml),
        Err(_) => Err(Error::ProcessFailed),
        Ok(_) => Err(Error::ValgrindFailure),
    }

    // TODO: use drop guard, that waits on child in order to prevent printing to stdout of the child
}
