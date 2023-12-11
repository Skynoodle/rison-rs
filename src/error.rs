//! Error and result types for Rison serialization and deserialization failures

use std::fmt::Write;

/// Categorizes an [`Error`]
#[derive(Debug)]
pub enum Category {
    /// failed to read or write bytes on an IO stream
    Io,
    /// input is not valid Rison
    Syntax,
    /// input that is not semantically correct
    Data,
    /// input that ended unexpectedly
    Eof,
}

#[derive(Debug)]
pub(crate) enum Code {
    Message(String),
    Io(std::io::Error),
    EofValue,
    EofList,
    EofObject,
    EofString,
    EofMarker,
    ExpectedColon,
    ExpectedListSepOrEnd,
    ExpectedObjectSepOrEnd,
    InvalidMarker,
    InvalidEscape,
    InvalidNumber,
    InvalidUnicode,
    TrailingChars,
}

/// An error that can occur while serializing or deserializing Rison
pub struct Error {
    pub(crate) code: Code,
    pub(crate) position: Option<usize>,
}

impl Error {
    /// Categorizes this error
    pub fn classify(&self) -> Category {
        match self.code {
            Code::Message(_) => Category::Data,
            Code::Io(_) => Category::Io,
            Code::EofValue
            | Code::EofList
            | Code::EofObject
            | Code::EofString
            | Code::EofMarker => Category::Eof,
            Code::ExpectedColon
            | Code::ExpectedListSepOrEnd
            | Code::ExpectedObjectSepOrEnd
            | Code::InvalidMarker
            | Code::InvalidEscape
            | Code::InvalidNumber
            | Code::InvalidUnicode
            | Code::TrailingChars => Category::Syntax,
        }
    }
    /// Zero-based position at which the error was detected
    ///
    /// Errors may currently be missing a position in some cases
    pub fn position(&self) -> Option<usize> {
        self.position
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Code::Message(msg) => f.write_str(msg),
            Code::Io(err) => err.fmt(f),
            Code::EofValue => f.write_str("EoF while parsing a value"),
            Code::EofList => f.write_str("EoF while parsing a list"),
            Code::EofObject => f.write_str("EoF while parsing an object"),
            Code::EofString => f.write_str("EoF while parsing a quoted string"),
            Code::EofMarker => f.write_str("EoF while parsing a `!` marker"),
            Code::ExpectedColon => f.write_str("expected `:`"),
            Code::ExpectedListSepOrEnd | Code::ExpectedObjectSepOrEnd => {
                f.write_str("expected `,` or `)`")
            }
            Code::InvalidMarker => f.write_str("invalid marker"),
            Code::InvalidEscape => f.write_str("invalid escape"),
            Code::InvalidNumber => f.write_str("invalid number"),
            Code::InvalidUnicode => f.write_str("invalid unicode code point"),
            Code::TrailingChars => f.write_str("trailing characters"),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error({:?}", self.code.to_string())?;
        if let Some(position) = self.position {
            write!(f, ", position: {}", position)?;
        }
        f.write_char(')')
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code.fmt(f)?;
        if let Some(position) = self.position {
            write!(f, " at position {}", position)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self {
            code: Code::Message(msg.to_string()),
            position: None,
        }
    }
}

/// An alias for [`Result`](std::result::Result) with the [`rison::Error`](Error) error type
pub type Result<T> = std::result::Result<T, Error>;
