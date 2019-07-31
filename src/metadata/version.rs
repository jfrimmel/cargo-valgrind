//! Deserializer for the `version` field of the metadata.
//!
//! This checks, that the metadata is indeed the version `1`, since any other
//! version cannot be reliably be parsed.
use serde::de::{Deserializer, Error, Visitor};
use std::fmt::{self, Formatter};

/// Deserialize the `version` field of the cargo metadata.
///
/// The only supported version is version 1, so any other value will fail to
/// deserialize.
pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_u32(VersionVisitor)
}

/// A macro for reducing the boilerplate of checking for `"1"` for each type.
macro_rules! visit_integer {
    ($function:ident, $type:ty) => {
        fn $function<E: Error>(self, value: $type) -> Result<Self::Value, E> {
            if value == 1 {
                Ok(value as _)
            } else {
                Err(E::custom("only version 1 is supported"))
            }
        }
    };
}

/// A visitor for the version field.
struct VersionVisitor;
impl<'de> Visitor<'de> for VersionVisitor {
    type Value = u32;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("1")
    }

    visit_integer!(visit_i8, i8);
    visit_integer!(visit_u8, u8);
    visit_integer!(visit_i16, i16);
    visit_integer!(visit_u16, u16);
    visit_integer!(visit_i32, i32);
    visit_integer!(visit_u32, u32);
    visit_integer!(visit_i64, i64);
    visit_integer!(visit_u64, u64);
}
