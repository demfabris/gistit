use console::style;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),

    #[error("{0}")]
    Request(#[from] reqwest::Error),

    #[error("{0}")]
    Clipboard(#[from] Clipboard),

    #[error("{0}")]
    Encoding(#[from] base64::DecodeError),

    #[error("{0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("{0}")]
    UrlParse(#[from] url::ParseError),

    #[error("{0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("{0}")]
    Ipc(#[from] gistit_ipc::Error),

    #[error("{0}")]
    Tui(#[from] bat::error::Error),

    #[error("{0}")]
    Other(#[from] which::Error),

    #[error("{0}")]
    Server(String),

    /// (Reason, Param)
    #[error("{}", fmt_subcat("PARAM", .0, .1))]
    Argument(&'static str, &'static str),

    #[error("{}", fmt_suggest("invalid colorscheme parameter", .0.clone()))]
    Colorscheme(String),

    #[error("{0}")]
    OAuth(String),

    #[error("unknown error")]
    Unknown,
}

fn fmt_suggest(cause: &'static str, suggest: String) -> String {
    format!(
        r#"{}

Did you mean: '{}'?
        "#,
        cause,
        style(suggest).blue().bold()
    )
}

fn fmt_subcat(subcat: &'static str, cause: &'static str, param: &'static str) -> String {
    format!(
        r#"{}

{}: 
    {}
"#,
        cause,
        subcat,
        style(param).dim()
    )
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

impl From<String> for Error {
    fn from(_: String) -> Self {
        Self::Unknown
    }
}
