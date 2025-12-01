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
/// Although there are also versions 1-3, there is only a variant for version 4
/// and newer, so that all older formats will fail. The other `struct`s in this
/// file assume the newer protocol versions, which are largely compatible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
enum ProtocolVersion {
    // older formats are not supported and won't be in the future
    /// Protocol [version 4].
    ///
    /// [version 4]: https://sourceware.org/git/?p=valgrind.git;a=blob_plain;f=docs/internals/xml-output-protocol4.txt;hb=d772e25995c3400eecf2b6070e0bf3411447c3d1
    #[serde(rename = "4")]
    Version4,
    /// Protocol [version 5].
    ///
    /// [version 5]: https://sourceware.org/git/?p=valgrind.git;a=blob_plain;f=docs/internals/xml-output-protocol5.txt;hb=48d64d0e6bb72220bb2557be4f427a57038dfbc6
    #[serde(rename = "5")]
    Version5,
    /// Protocol [version 6].
    ///
    /// [version 6]: https://sourceware.org/git/?p=valgrind.git;a=blob_plain;f=docs/internals/xml-output-protocol6.txt;hb=7786b075abef51ca3d84b9717915f04b32950b32
    #[serde(rename = "6")]
    Version6,
    // newer versions are not yet supported! Feel free to add one via a PR.
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
    #[serde(default)]
    #[serde(rename = "xwhat")]
    pub resources: Resources,
    #[serde(default)]
    #[serde(rename = "what")]
    pub main_info: Option<String>,
    #[serde(default)]
    #[serde(rename = "auxwhat")]
    pub auxiliary_info: Vec<String>,
    #[serde(rename = "stack")]
    pub stack_trace: Vec<Stack>,
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
    FdBadUse,
    ClientCheck,
}
impl Kind {
    /// Query, if the current error kind is a memory leak
    pub(crate) const fn is_leak(self) -> bool {
        match self {
            Self::LeakDefinitelyLost
            | Self::LeakStillReachable
            | Self::LeakIndirectlyLost
            | Self::LeakPossiblyLost => true,
            Self::InvalidFree
            | Self::MismatchedFree
            | Self::InvalidRead
            | Self::InvalidWrite
            | Self::InvalidJump
            | Self::Overlap
            | Self::InvalidMemPool
            | Self::UninitCondition
            | Self::UninitValue
            | Self::SyscallParam
            | Self::FdBadUse
            | Self::ClientCheck => false,
        }
    }
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
            Self::FdBadUse => write!(f, "bad file descriptor use"),
            Self::ClientCheck => write!(f, "client check"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
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
        f.write_str(self.function.as_ref().map_or("unknown", |s| s.as_str()))?;
        if let Some(file) = &self.file {
            f.write_str(" (")?;
            f.write_str(file)?;
            if let Some(line) = self.line {
                write!(f, ":{line}")?;
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
impl Visitor<'_> for HexVisitor {
    type Value = u64;

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("hexadecimal number with leading '0x'")
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        let value = value.to_ascii_lowercase();
        let value = value
            .strip_prefix("0x")
            .ok_or_else(|| E::custom("'0x' prefix missing"))?;
        Self::Value::from_str_radix(value, 16)
            .map_err(|_| E::custom(format!("invalid hex number '{value}'")))
    }
}
