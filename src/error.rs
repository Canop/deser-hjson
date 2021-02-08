use {
    serde::de,
    std::fmt,
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

    /// a Hjson syntax error raised in our code,
    /// with location
    Syntax {
        line: usize,
        col: usize, // in chars (tab is one char)
        code: ErrorCode,
        at: String, // next few chars
    },

    /// A Serde error, with approximate location
    Serde {
        line: usize,
        col: usize, // in chars (tab is one char)
        message: String,
    },

    /// a raw Serde error. We should try to
    /// convert them to Serde located errors as
    /// much as possible
    RawSerde(String),
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::RawSerde(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Syntax { line, col, code, at } => {
                write!(formatter, "{:?} at {}:{} at {:?}", code, line, col, at)
            }
            Self::Serde { line, col, message } => {
                write!(formatter, "{:?} near {}:{}", message, line, col)
            }
            Self::RawSerde(msg) => {
                write!(formatter, "error message: {:?}", msg)
            }
        }
    }
}

impl std::error::Error for Error {}
