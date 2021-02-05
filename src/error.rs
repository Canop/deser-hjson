use {
    serde::{de, ser},
    std::{
        self,
        fmt::{self, Display},
    },
};

pub type Result<T> = std::result::Result<T, Error>;

/// The types of errors which can happen in our code
/// during deserialization
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCode {
    Eof,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedI8,
    ExpectedI16,
    ExpectedI32,
    ExpectedI64,
    ExpectedU8,
    ExpectedU16,
    ExpectedU32,
    ExpectedU64,
    ExpectedF32,
    ExpectedF64,
    ExpectedPositiveInteger,
    ExpectedString,
    ExpectedNull,
    ExpectedArray,
    ExpectedArrayComma,
    ExpectedArrayEnd,
    ExpectedMap,
    ExpectedMapColon,
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedEnum,
    ExpectedSingleChar,
    InvalidEscapeSequence,
    TrailingCharacters,
    UnexpectedChar,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Syntax {
        line: usize,
        col: usize, // in chars (tab is one char)
        code: ErrorCode,
        at: String, // next few chars
    },
    Message(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Syntax { line, col, code, at } => {
                write!(formatter, "{:?} at {}:{} at {:?}", code, line, col, at)
            }
            Self::Message(msg) => {
                write!(formatter, "error message: {:?}", msg)
            }
        }
    }
}

impl std::error::Error for Error {}
