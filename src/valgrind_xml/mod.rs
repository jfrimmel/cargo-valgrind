//! A module containing the structure of the valgrind XML output.
//!
//! Only the memcheck tool is implemented in accordance to [this][link]
//! description].
//!
//! Note, that not all fields are implemented.
//!
//! [link]: https://github.com/fredericgermain/valgrind/blob/master/docs/internals/xml-output-protocol4.txt
#![allow(clippy::missing_docs_in_private_items)]

#[cfg(test)]
mod tests;

use serde::{de::Visitor, Deserialize, Deserializer};
use std::fmt::{self, Display, Formatter};

/// The output of a valgrind run.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename = "valgrindoutput")]
pub struct Output {
    #[serde(rename = "protocolversion")]
    protocol_version: ProtocolVersion,
    #[serde(rename = "protocoltool")]
    tool: Tool,
    #[serde(rename = "error")]
    pub errors: Option<Vec<Error>>,
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
pub struct Error {
    #[serde(deserialize_with = "deserialize_hex")]
    unique: u64,
    pub kind: Kind,
    #[serde(rename = "xwhat")]
    pub resources: Resources,
    #[serde(rename = "stack")]
    pub stack_trace: Stack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Kind {
    #[serde(rename = "Leak_DefinitelyLost")]
    LeakDefinitelyLost,
    #[serde(rename = "Leak_StillReachable")]
    LeakStillReachable,
    #[serde(rename = "Leak_IndirectlyLost")]
    LeakIndirectlyLost,
    #[serde(rename = "Leak_PossiblyLost")]
    LeakPossiblyLost,
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
impl Display for Kind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::LeakDefinitelyLost => write!(f, "Leak (definitely lost)"),
            Self::LeakStillReachable => write!(f, "Leak (still reachable)"),
            Self::LeakIndirectlyLost => write!(f, "Leak (indirectly lost)"),
            Self::LeakPossiblyLost => write!(f, "Leak (possibly lost)"),
            Self::InvalidFree => write!(f, "invalid free"),
            Self::MismatchedFree => write!(f, "mismatched free"),
            Self::InvalidRead => write!(f, "invalid read"),
            Self::InvalidWrite => write!(f, "invalid write"),
            Self::InvalidJump => write!(f, "invalid jump"),
            Self::Overlap => write!(f, "overlap"),
            Self::InvalidMemPool => write!(f, "invalid memory pool"),
            Self::UninitCondition => write!(f, "uninitialized condition"),
            Self::UninitValue => write!(f, "uninitialized value"),
            Self::SyscallParam => write!(f, "syscall parameter"),
            Self::ClientCheck => write!(f, "client check"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Resources {
    #[serde(rename = "leakedbytes")]
    pub bytes: usize,
    #[serde(rename = "leakedblocks")]
    pub blocks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Stack {
    #[serde(rename = "frame")]
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Frame {
    #[serde(rename = "ip")]
    #[serde(deserialize_with = "deserialize_hex")]
    pub instruction_pointer: u64,
    #[serde(rename = "obj")]
    pub object: Option<String>,
    #[serde(rename = "dir")]
    pub directory: Option<String>,
    #[serde(rename = "fn")]
    pub function: Option<String>,
    pub file: Option<String>,
    pub line: Option<usize>,
}
impl Display for Frame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.function.as_ref().unwrap_or(&"unknown".into()))?;
        if let Some(file) = &self.file {
            f.write_str(" (")?;
            f.write_str(file)?;
            if let Some(line) = self.line {
                write!(f, ":{}", line)?;
            }
            f.write_str(")")?;
        }
        Ok(())
    }
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
