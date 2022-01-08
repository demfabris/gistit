#[derive(Debug)]
pub enum ErrorKind {
    InvalidArgs,
    FsEvent(String),
    Io(std::io::Error),
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl From<notify::Error> for Error {
    fn from(err: notify::Error) -> Self {
        Self {
            kind: ErrorKind::FsEvent(err.to_string()),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(err),
        }
    }
}
