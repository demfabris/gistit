use console::style;

impl From<String> for Error {
    fn from(cause: String) -> Self {
        Self {
            cause: Box::leak(Box::new(cause)),
            kind: ErrorKind::Unknown,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        let cause = Box::leak(Box::new(err.to_string()));
        Self {
            cause,
            kind: ErrorKind::IO(err),
        }
    }
}

impl From<crate::Settings> for Error {
    fn from(_: crate::Settings) -> Self {
        Self {
            cause: "failed to parse settings.",
            kind: ErrorKind::Settings,
        }
    }
}

impl From<lib_gistit::Error> for Error {
    fn from(err: lib_gistit::Error) -> Self {
        Self {
            cause: "internal error.",
            kind: ErrorKind::Internal(err),
        }
    }
}

impl From<url::ParseError> for Error {
    fn from(_: url::ParseError) -> Self {
        Self {
            cause: "failed to parse url.",
            kind: ErrorKind::Parsing,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self {
            cause: Box::leak(Box::new(err.to_string())),
            kind: ErrorKind::Request(err),
        }
    }
}

impl From<bat::error::Error> for Error {
    fn from(err: bat::error::Error) -> Self {
        Self {
            kind: ErrorKind::Tui(err),
            cause: "failed to print to stdout.",
        }
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self {
            kind: ErrorKind::SerializeYaml(err),
            cause: "failed to parse yaml file.",
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(std::io::Error),
    Internal(lib_gistit::Error),
    Request(reqwest::Error),
    Tui(bat::error::Error),
    SerializeYaml(serde_yaml::Error),

    Colorscheme(Option<String>),
    InvalidParam(&'static str, &'static str),

    FetchNotFound,
    FetchUnexpectedResponse,
    FetchEnoughRetries,

    Parsing,
    Argument,
    Settings,
    Unknown,
}

pub struct Error {
    pub kind: ErrorKind,
    pub cause: &'static str,
}

fn s_string(string: String) -> &'static str {
    Box::leak(Box::new(string))
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        match kind {
            ErrorKind::IO(e) => e.into(),
            ErrorKind::Internal(e) => e.into(),
            ErrorKind::Request(e) => e.into(),
            ErrorKind::Tui(e) => e.into(),
            ErrorKind::SerializeYaml(e) => e.into(),
            ErrorKind::Colorscheme(ref maybe_close_match) => {
                let suggest = maybe_close_match
                    .as_ref()
                    .map(|close_match| format!("\ndid you mean: '{}'?", style(close_match).blue()));
                Self {
                    kind,
                    cause: s_string(format!(
                        "invalid colorscheme parameter.{}",
                        suggest.unwrap_or_else(|| "".to_owned())
                    )),
                }
            }
            ErrorKind::InvalidParam(msg, param) => Self {
                kind,
                cause: s_string(format!("{}\n{}", msg, style(param).red())),
            },
            ErrorKind::FetchNotFound => Self {
                kind,
                cause: "no gistit were found with this hash.",
            },
            ErrorKind::FetchUnexpectedResponse => Self {
                kind,
                cause: "got an unexpected response during fetch.",
            },
            ErrorKind::FetchEnoughRetries => Self {
                kind,
                cause: "invalid password entered to many times.",
            },
            ErrorKind::Parsing => Self {
                kind,
                cause: "failed to parse argument",
            },
            ErrorKind::Argument => Self {
                kind,
                cause: "failed to parse argument",
            },
            ErrorKind::Settings => Self {
                kind,
                cause: "failed to parse argument",
            },
            ErrorKind::Unknown => Self {
                kind,
                cause: "failed to parse argument",
            },
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"
CAUSE:
    {}
            "#,
            self.cause
        )
    }
}
