pub mod de;

mod read;

#[derive(Debug)]
enum ErrorKind {
    Custom(String),
    Io(std::io::Error),
    Syntax,
    Eof,
    Semantic,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::Custom(custom) => {
                "custom: ".fmt(f)?;
                custom.fmt(f)
            }
            ErrorKind::Io(e) => write!(f, "io error: {e}"),
            ErrorKind::Syntax => "syntax error".fmt(f),
            ErrorKind::Eof => "unexpected eof".fmt(f),
            ErrorKind::Semantic => "semantic error".fmt(f),
        }
    }
}

// impl std::fmt::Debug for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Error").finish()
//     }
// }
impl std::error::Error for Error {}
impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self {
            kind: ErrorKind::Custom(msg.to_string()),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;
