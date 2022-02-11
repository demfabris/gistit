#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(std::io::Error),
    Serialization(bincode::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::IO(err),
        }
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Self {
            kind: ErrorKind::Serialization(err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "gistit ipc error"
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gistit ipc error")
    }
}
