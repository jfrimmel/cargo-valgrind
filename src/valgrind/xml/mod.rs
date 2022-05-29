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

use std::fmt::{self, Display, Formatter};
use strong_xml::{XmlError, XmlRead, XmlReader, XmlResult};
use strum::EnumString;

/// The output of a valgrind run.
#[derive(Debug, Clone, PartialEq, Eq, Hash, XmlRead)]
#[xml(tag = "valgrindoutput")]
pub struct Output {
    #[xml(flatten_text = "protocolversion")]
    protocol_version: ProtocolVersion,
    #[xml(flatten_text = "protocoltool")]
    tool: Tool,
    #[xml(child = "error")]
    pub errors: Vec<Error>,
}

/// The version of the XML format.
///
/// Although there are also versions 1-3, there is only a variant for version 4,
/// so that all older formats will fail. The other `struct`s in this file assume
/// the newest protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString)]
enum ProtocolVersion {
    #[strum(serialize = "4")]
    Version4,
    // other formats are not supported
}

/// The check tool used by valgrind.
///
/// Although there are other tools available, there is only a variant for the
/// so-called `memcheck` tool, so that all other tools will fail. The other
/// `struct`s in this file assume the memcheck output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString)]
enum Tool {
    #[strum(serialize = "memcheck")]
    MemCheck,
    // other tools are not supported
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    unique: u64,
    pub kind: Kind,
    pub description: String,
    pub resources: Resources,
    pub stack_trace: Stack,
    pub extra: Vec<ErrorExtra>,
}
impl<'a> XmlRead<'a> for Error {
    fn from_reader(reader: &mut XmlReader<'a>) -> XmlResult<Self> {
        #[derive(XmlRead)]
        enum What {
            #[xml(tag = "what")]
            What(#[xml(text)] String),
            #[xml(tag = "xwhat")]
            XWhat {
                #[xml(flatten_text = "text")]
                text: String,
                #[xml(flatten_text = "leakedbytes", default)]
                bytes: usize,
                #[xml(flatten_text = "leakedblocks", default)]
                blocks: usize,
            },
        }

        #[derive(XmlRead)]
        #[xml(tag = "error")]
        struct TmpError {
            #[xml(flatten_text = "unique")]
            unique: Hex64,
            #[xml(flatten_text = "kind")]
            pub kind: Kind,
            #[xml(child = "what", child = "xwhat")]
            pub what: What,
            #[xml(child = "stack", child = "auxwhat", child = "xauxwhat")]
            pub extra: Vec<ErrorExtra>,
        }

        let TmpError {
            unique: Hex64(unique),
            kind,
            what,
            mut extra,
        } = TmpError::from_reader(reader)?;
        let first = (!extra.is_empty()).then(|| extra.remove(0));
        let stack_trace = if let Some(ErrorExtra::StackTrace(s)) = first {
            s
        } else {
            return Err(XmlError::MissingField {
                name: "Error".to_string(),
                field: "stack_trace".to_string(),
            });
        };
        let (description, resources) = match what {
            What::What(s) => (s, Resources::default()),
            What::XWhat {
                text,
                bytes,
                blocks,
            } => (text, Resources { bytes, blocks }),
        };
        Ok(Self {
            unique,
            kind,
            description,
            resources,
            stack_trace,
            extra,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorExtra {
    StackTrace(Stack),
    AuxWhat(String),
}
impl<'a> XmlRead<'a> for ErrorExtra {
    fn from_reader(reader: &mut XmlReader<'a>) -> XmlResult<Self> {
        #[derive(XmlRead)]
        pub enum TmpErrorExtra {
            #[xml(tag = "stack")]
            StackTrace(Stack),
            #[xml(tag = "auxwhat")]
            AuxWhat(#[xml(text)] String),
            #[xml(tag = "xauxwhat")]
            XAuxWhat(#[xml(flatten_text = "text")] String),
        }

        Ok(match TmpErrorExtra::from_reader(reader)? {
            TmpErrorExtra::StackTrace(s) => Self::StackTrace(s),
            TmpErrorExtra::AuxWhat(s) | TmpErrorExtra::XAuxWhat(s) => Self::AuxWhat(s),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString)]
pub enum Kind {
    #[strum(serialize = "Leak_DefinitelyLost")]
    LeakDefinitelyLost,
    #[strum(serialize = "Leak_StillReachable")]
    LeakStillReachable,
    #[strum(serialize = "Leak_IndirectlyLost")]
    LeakIndirectlyLost,
    #[strum(serialize = "Leak_PossiblyLost")]
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resources {
    pub bytes: usize,
    pub blocks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, XmlRead)]
#[xml(tag = "stack")]
pub struct Stack {
    #[xml(child = "frame")]
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Frame {
    pub instruction_pointer: u64,
    pub object: Option<String>,
    pub directory: Option<String>,
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
impl<'a> XmlRead<'a> for Frame {
    fn from_reader(reader: &mut XmlReader<'a>) -> XmlResult<Self> {
        #[derive(XmlRead)]
        #[xml(tag = "frame")]
        pub struct TmpFrame {
            #[xml(flatten_text = "ip")]
            pub instruction_pointer: Hex64,
            #[xml(flatten_text = "obj")]
            pub object: Option<String>,
            #[xml(flatten_text = "dir")]
            pub directory: Option<String>,
            #[xml(flatten_text = "fn")]
            pub function: Option<String>,
            #[xml(flatten_text = "file")]
            pub file: Option<String>,
            #[xml(flatten_text = "line")]
            pub line: Option<usize>,
        }

        let TmpFrame {
            instruction_pointer: Hex64(instruction_pointer),
            object,
            directory,
            function,
            file,
            line,
        } = TmpFrame::from_reader(reader)?;
        Ok(Self {
            instruction_pointer,
            object,
            directory,
            function,
            file,
            line,
        })
    }
}

use hex64::Hex64;
mod hex64 {
    use std::error::Error;
    use std::fmt::{self, Display, Formatter};
    use std::num::ParseIntError;
    use std::str::FromStr;

    #[derive(Debug)]
    pub enum ParseHex64Error {
        MissingPrefix,
        ParseInt(ParseIntError),
    }

    impl Display for ParseHex64Error {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Self::MissingPrefix => write!(f, "HEX64: missing 0x prefix"),
                Self::ParseInt(err) => write!(f, "HEX64: {}", err),
            }
        }
    }

    impl Error for ParseHex64Error {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::MissingPrefix => None,
                Self::ParseInt(err) => Some(err),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Hex64(pub u64);
    impl FromStr for Hex64 {
        type Err = ParseHex64Error;

        fn from_str(value: &str) -> Result<Self, Self::Err> {
            let value = value.to_ascii_lowercase();
            let value = value
                .strip_prefix("0x")
                .ok_or(ParseHex64Error::MissingPrefix)?;
            u64::from_str_radix(value, 16)
                .map(Hex64)
                .map_err(ParseHex64Error::ParseInt)
        }
    }
}
