use console::style;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{}", fmt(.0))]
    IO(#[from] std::io::Error),

    #[error("{}", fmt(.0))]
    Request(#[from] reqwest::Error),

    #[error("{}", fmt(.0))]
    Clipboard(#[from] Clipboard),

    #[error("{}", fmt(.0))]
    Encoding(#[from] base64::DecodeError),

    #[error("{}", fmt(.0))]
    Ipc(#[from] gistit_ipc::Error),

    #[error("{}", fmt(.0))]
    Tui(#[from] bat::error::Error),

    #[error("{}", fmt(.0))]
    Other(#[from] which::Error),

    #[error("{}", fmt(Self::from(.0.clone())))]
    Server(String),

    /// (Reason, Param)
    #[error("{}", fmt_arg(.0, .1))]
    Argument(&'static str, &'static str),

    #[error("{}", fmt_suggest("invalid colorscheme parameter", .0.clone()))]
    Colorscheme(String),

    #[error("{}", fmt(Self::from("unknown cause".to_owned())))]
    Unknown,
}

fn fmt(err: impl std::error::Error) -> String {
    format!(
        r#"
CAUSE:
    {}
        "#,
        err
    )
}

fn fmt_suggest(cause: &'static str, suggest: String) -> String {
    format!(
        r#"
CAUSE:
    {}

Did you mean: {}?
        "#,
        cause,
        style(suggest).blue().bold()
    )
}

fn fmt_arg(cause: &'static str, param: &'static str) -> String {
    format!(
        r#"
CAUSE:
    {}

PARAM:
    {}
"#,
        cause,
        style(param).red().bold()
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
