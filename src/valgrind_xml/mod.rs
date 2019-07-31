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

use serde::{Deserialize, Deserializer, de::Visitor};
use std::fmt::{self, Formatter};

/// The output of a valgrind run.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename = "valgrindoutput")]
pub struct Output {
    #[serde(rename = "error")]
    errors: Vec<Error>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct Error {
    #[serde(deserialize_with = "deserialize_hex")]
    unique: u64,
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

fn deserialize_hex<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    deserializer.deserialize_str(HexVisitor)
}

struct HexVisitor;
impl<'de> Visitor<'de> for HexVisitor {
    type Value = u64;

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("hexadecimal number with leading '0x'")
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        let value = value.to_ascii_lowercase();
        if value.starts_with("0x") {
            value[2..].parse().map_err(|_| E::custom(format!("invalid hex number")))
        } else {
            Err(E::custom("'0x' prefix missing"))
        }
    }
}