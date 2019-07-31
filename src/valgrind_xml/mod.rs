//! A module containing the structure of the valgrind XML output.
//!
//! Note, that not all fields are implemented.
#[cfg(test)]
mod tests;

use serde::Deserialize;

/// The output of a valgrind run.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename = "valgrindoutput")]
pub struct Output {
    #[serde(rename = "error")]
    errors: Vec<Error>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct Error {
    kind: Kind,
    #[serde(rename = "xwhat")]
    resources: Resources,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
enum Kind {
    #[serde(rename = "Leak_DefinitelyLost")]
    DefinitelyLost,
    #[serde(rename = "Leak_StillReachable")]
    StillReachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
struct Resources {
    #[serde(rename = "leakedbytes")]
    bytes: usize,
    #[serde(rename = "leakedblocks")]
    blocks: usize,
}
