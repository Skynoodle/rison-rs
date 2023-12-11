pub enum Category {
    Io,
    Syntax,
    Data,
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

#[derive(Debug)]
pub struct Error {
    pub(crate) code: Code,
}

impl Error {
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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.code {
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

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self {
            code: Code::Message(msg.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
