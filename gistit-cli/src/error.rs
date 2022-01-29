#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error, {0}")]
    IO(#[from] std::io::Error),

    #[error("request error, {0}")]
    Request(#[from] reqwest::Error),

    #[error("clipboard error, {0}")]
    Clipboard(#[from] Clipboard),

    #[error("decoding error, {0}")]
    Encoding(#[from] base64::DecodeError),

    #[error("ipc error, {0}")]
    Ipc(#[from] gistit_ipc::Error),

    #[error("other error")]
    Other(#[from] which::Error),

    #[error("server error, {0}")]
    Server(String),

    #[error("failed to print to terminal")]
    Tui(#[from] bat::error::Error),

    /// (Reason, Param)
    #[error("argument error, {0}")]
    Argument(&'static str, &'static str),

    #[error("invalid colorscheme, did you mean: {0}?")]
    Colorscheme(String),

    #[error("unknown error")]
    Unknown,
}

#[derive(thiserror::Error, Debug)]
pub enum Clipboard {
    #[error("this platform is not supported")]
    UnsupportedPlatform,
    #[error("couldn't find any supported clipboard binaries")]
    MissingBinary,
    #[error("the environment variable `DISPLAY` is not set")]
    DisplayNotSet,
}
