#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub cause: &'static str,
}

#[derive(Debug)]
pub enum ErrorKind {
    Encoding(base64::DecodeError),
    IO(std::io::Error),
    NotFound(which::Error),
    EncryptionPadding,
    FileExtension,
    FileSize,
    NotAFile,
    UnsupportedPlatform,
    MissingClipboardBinary,
    DisplayNotSet,
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        match kind {
            ErrorKind::Encoding(e) => e.into(),
            ErrorKind::IO(e) => e.into(),
            ErrorKind::NotFound(e) => e.into(),
            ErrorKind::EncryptionPadding => Self {
                kind,
                cause: "unable to parse encrypted data, nonce or padding are missplaced.",
            },
            ErrorKind::FileExtension => Self {
                kind,
                cause: "file extension not currently supported.",
            },
            ErrorKind::FileSize => Self {
                kind,
                cause: "file size not allowed.",
            },
            ErrorKind::NotAFile => Self {
                kind,
                cause: "input is not a file",
            },
            ErrorKind::UnsupportedPlatform => Self {
                kind,
                cause: "could not enable clipboard feature under this platform.
supported platforms: Linux, BSD, Windows, MacOs.",
            },
            ErrorKind::MissingClipboardBinary => Self {
                kind,
                cause: "could not find any clipboard program binaries. ('xclip', 'xsel', 'wl-copy', 'pbcopy')
please consider installing of the above programs to have more reliable results.",
            },
            ErrorKind::DisplayNotSet => Self {
                kind,
                cause: "enviroment variable 'DISPLAY' is not set.
clipboard is likely not working",
            }
        }
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Self {
            kind: ErrorKind::Encoding(err),
            cause: "the encoding/decoding process failed, data might be inconsistent/corrupted.",
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        let cause = Box::leak(Box::new(err.to_string()));
        Self {
            kind: ErrorKind::IO(err),
            cause,
        }
    }
}

impl From<which::Error> for Error {
    fn from(err: which::Error) -> Self {
        Self {
            kind: ErrorKind::NotFound(err),
            cause: "program not installed.",
        }
    }
}
