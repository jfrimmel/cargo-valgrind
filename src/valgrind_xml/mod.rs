//! A module containing the structure of the valgrind XML output.
//!
//! Only the memcheck tool is implemented in accordance to [this][link]
//! description].
//!
//! Note, that not all fields are implemented.
//!
//! [link]: https://github.com/fredericgermain/valgrind/blob/master/docs/internals/xml-output-protocol4.txt
#[cfg(test)]
mod tests;

use serde::{de::Visitor, Deserialize, Deserializer};
use std::fmt::{self, Formatter};

/// The output of a valgrind run.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename = "valgrindoutput")]
pub struct Output {
    #[serde(rename = "protocolversion")]
    protocol_version: ProtocolVersion,
    #[serde(rename = "protocoltool")]
    tool: Tool,
    #[serde(rename = "error")]
    errors: Vec<Error>,
}

/// The version of the XML format.
///
/// Although there are also versions 1-3, there is only a variant for version 4,
/// so that all older formats will fail. The other `struct`s in this file assume
/// the newest protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
enum ProtocolVersion {
    #[serde(rename = "4")]
    Version4,
    // other formats are not supported
}

/// The check tool used by valgrind.
///
/// Although there are other tools available, there is only a variant for the
/// so-called `memcheck` tool, so that all other tools will fail. The other
/// `struct`s in this file assume the memcheck output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
enum Tool {
    #[serde(rename = "memcheck")]
    MemCheck,
    // other tools are not supported
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct Error {
    #[serde(deserialize_with = "deserialize_hex")]
    unique: u64,
    kind: Kind,
    #[serde(rename = "xwhat")]
    resources: Resources,
    #[serde(rename = "stack")]
    stack_trace: Stack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
enum Kind {
    #[serde(rename = "Leak_DefinitelyLost")]
    DefinitelyLost,
    #[serde(rename = "Leak_StillReachable")]
    StillReachable,
    #[serde(rename = "Leak_IndirectlyLost")]
    IndirectlyLost,
    #[serde(rename = "Leak_PossiblyLost")]
    PossiblyLost,
    InvalidFree,
    MismatchedFree,
    InvalidRead,
    InvalidWrite,
    InvalidJump,
    Overlap,
    InvalidMemPool,
    UninitCondition,
    UninitValue,
    SyscallParam,
    ClientCheck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
struct Resources {
    #[serde(rename = "leakedbytes")]
    bytes: usize,
    #[serde(rename = "leakedblocks")]
    blocks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct Stack {
    #[serde(rename = "frame")]
    frames: Vec<Frame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct Frame {
    #[serde(rename = "ip")]
    #[serde(deserialize_with = "deserialize_hex")]
    instruction_pointer: u64,
    #[serde(rename = "obj")]
    object: Option<String>,
    #[serde(rename = "dir")]
    directory: Option<String>,
    #[serde(rename = "fn")]
    function: Option<String>,
    file: Option<String>,
    line: Option<usize>,
}

fn deserialize_hex<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    deserializer.deserialize_str(HexVisitor)
}

/// A visitor for parsing a `u64` in the format `0xDEADBEEF`.
struct HexVisitor;
impl<'de> Visitor<'de> for HexVisitor {
    type Value = u64;

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("hexadecimal number with leading '0x'")
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        let value = value.to_ascii_lowercase();
        if value.starts_with("0x") {
            Self::Value::from_str_radix(&value[2..], 16)
                .map_err(|_| E::custom(format!("invalid hex number '{}'", value)))
        } else {
            Err(E::custom("'0x' prefix missing"))
        }
    }
}
