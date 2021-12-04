//! Errors module
//!
//! Here we define all errors (fatal and non-fatal) that gistit can output
//! We moved to manual implementation of the errors to allow for prettier and colorized output
//! Every error should convert to top level [`Error`] in the end.
use console::{style, Emoji};

/// The top level error structure
pub enum Error {
    /// File reading errors
    File(file::FileError),
    /// Clipboard feature errors
    #[cfg(feature = "clipboard")]
    Clipboard(clipboard::ClipboardError),
    /// Gistit params errors
    Params(params::ParamsError),
    /// I/O operations errors
    IO(io::IoError),
    /// File encrypting/hashing errors
    Encryption(encryption::EncryptionError),
    /// Gistit-fetch related errors
    Fetch(fetch::FetchError),
    /// Argument parsing errors
    Argument,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::File(err) => {
                write!(f, "{}", err)
            }
            Self::Clipboard(err) => {
                write!(f, "{}", err)
            }
            Self::Params(err) => {
                write!(f, "{}", err)
            }
            Self::Encryption(err) => {
                write!(f, "{}", err)
            }
            Self::IO(err) => {
                write!(f, "{}", err)
            }
            Self::Fetch(err) => {
                write!(f, "{}", err)
            }
            Self::Argument => {
                write!(f, "Something went wrong during arg parsing")
            }
        }
    }
}

/// Encryption module errors
pub mod encryption {
    use base64::DecodeError;

    use super::{style, Emoji, Error};

    pub enum EncryptionError {
        SecretLength,
        /// Errors related to ciphering. Output from 'rust-crypto' crate symmetric cipher ops
        Cipher(aes_gcm::Error),
        /// Base64 de/encode errors
        Encoding(DecodeError),
    }

    impl From<EncryptionError> for Error {
        fn from(err: EncryptionError) -> Self {
            Self::Encryption(err)
        }
    }

    impl From<aes_gcm::Error> for Error {
        fn from(err: aes_gcm::Error) -> Self {
            Self::Encryption(EncryptionError::Cipher(err))
        }
    }

    impl From<DecodeError> for Error {
        fn from(err: DecodeError) -> Self {
            Self::Encryption(EncryptionError::Encoding(err))
        }
    }

    impl std::fmt::Display for EncryptionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                EncryptionError::SecretLength => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("SecretLength").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    invalid secret character length.
    min = {} max = {}
                    "#,
                        style("--secret <secret>").red().bold(),
                        style("5 chars").yellow(),
                        style("50 chars").yellow(),
                    )
                }
                EncryptionError::Cipher(err) => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("Cipher").red().bold()
                    );
                    write!(
                        f,
                        r#"
CAUSE:
    the encryption process failed.
    {:?}

This is unlikely to be caused by a misuse of the application, check your program version.
                    "#,
                        style(err).yellow()
                    )
                }
                EncryptionError::Encoding(err) => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("Encoding").red().bold()
                    );
                    write!(
                        f,
                        r#"
CAUSE:
    the encoding/decoding process failed, data might be inconsistent/corrupted.
    {:?}

This is unlikely to be caused by a misuse of the application, check your program version.
                    "#,
                        style(err).yellow()
                    )
                }
            }
        }
    }
}

/// Params module errors
pub mod params {
    use super::{style, Emoji, Error};

    pub enum ParamsError {
        DescriptionCharRange,
        AuthorCharRange,
        Colorscheme(Option<String>),
        LifespanRange,
        InvalidLifespan,
        InvalidUrl(String),
        InvalidHash(String),
    }

    impl From<ParamsError> for Error {
        fn from(err: ParamsError) -> Self {
            Self::Params(err)
        }
    }

    impl std::fmt::Display for ParamsError {
        #[allow(clippy::too_many_lines)]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                ParamsError::DescriptionCharRange => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("DescriptionCharLength").red().bold()
                    );
                    write!(
                        f,
                        r#"
CAUSE:
    invalid description character length.
    min = {} max = {}
                    "#,
                        style("10 chars").yellow(),
                        style("100 chars").yellow()
                    )
                }
                ParamsError::AuthorCharRange => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("AuthorCharRange").red().bold()
                    );
                    write!(
                        f,
                        r#"
CAUSE:
    invalid author character length.
    min = {} max = {}
                    "#,
                        style("3 chars").yellow(),
                        style("30 chars").yellow()
                    )
                }
                ParamsError::Colorscheme(maybe_close_match) => {
                    let suggest = maybe_close_match.as_ref().map(|close_match| {
                        format!("\n\nDid you mean: '{}'?", style(close_match).blue())
                    });
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("Colorscheme").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    invalid colorscheme parameter.
    run '{}' to list supported colorschemes.{}
                    "#,
                        style("--colorscheme <colorscheme>").red().bold(),
                        style("gistit --colorschemes").bold(),
                        suggest.unwrap_or_else(|| "".to_string())
                    )
                }
                ParamsError::LifespanRange => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("LifespanRange").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    invalid lifespan parameter value range.
    min = {} max = {}
                    "#,
                        style("--lifespan <lifespan>").red().bold(),
                        style("300s").yellow(),
                        style("3600s (default)").yellow()
                    )
                }
                ParamsError::InvalidLifespan => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("InvalidLifespan").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    invalid lifespan parameter, input is not a positive number.
                    "#,
                        style("--lifespan <lifespan>").red().bold(),
                    )
                }
                ParamsError::InvalidUrl(err) => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("InvalidUrl").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    input is not a valid URL.
    {}
                    "#,
                        style("--url <url>").red().bold(),
                        style(err).yellow()
                    )
                }
                ParamsError::InvalidHash(hash_captured) => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("InvalidHash").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    input "{}" is not a valid gistit hash.
                    "#,
                        style("--hash <hash>").red().bold(),
                        style(hash_captured).yellow()
                    )
                }
            }
        }
    }
}

/// File module errors
pub mod file {
    use super::{style, Emoji, Error};

    #[derive(Clone)]
    pub enum FileError {
        /// File extension doesn't match supported ones
        UnsupportedExtension(String),
        /// File size is outside allowed range
        UnsupportedSize(u64),
        /// File is not a file
        NotAFile(String),
        /// File has weird extensions
        MissingExtension,
        /// Invalid embedded hmac/padding
        InvalidEncryptionPadding,
    }

    impl From<FileError> for Error {
        fn from(err: FileError) -> Self {
            Self::File(err)
        }
    }

    impl std::fmt::Display for FileError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Self::MissingExtension => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("MissingFileExtension").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    input file must have an extension.
    see supported file extensions here: {}
                    "#,
                        style("--file <file>").red().bold(),
                        style("[redacted]").blue()
                    )
                }
                Self::UnsupportedExtension(ext) => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("UnsupportedExtension").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    file extension not currently supported: '{}'
    see supported file extensions here: {}
                    "#,
                        style("--file <file>").red().bold(),
                        style(ext).yellow(),
                        style("[redacted]").blue()
                    )
                }
                Self::NotAFile(name) => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("NotAFile").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    input '{}' is not a file
    "#,
                        style("--file <file>").red().bold(),
                        style(name).yellow()
                    )
                }
                Self::UnsupportedSize(size) => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("UnsupportedSize").red().bold()
                    );
                    let size_str = if size > &1 {
                        format!("{} bytes", size.to_string())
                    } else {
                        format!("{} byte", size.to_string())
                    };
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    file size is not in allowed range. ({})
    min = {} max = {}
                    "#,
                        style("--file <file>").red().bold(),
                        style(size_str).red().bold(),
                        style("20 B").yellow(),
                        style("50 KiB").yellow()
                    )
                }
                Self::InvalidEncryptionPadding => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("InvalidEncryptionHeader").red().bold()
                    );
                    write!(
                        f,
                        r#"
CAUSE:
    unable to parse encrypted data, nounce or padding are missplaced.
                    "#,
                    )
                }
            }
        }
    }
}

/// I/O operations error
pub mod io {
    use super::{style, Emoji, Error};

    #[derive(Clone)]
    pub enum IoError {
        /// Failed to spawn a process
        ProcessSpawn(String),
        /// Failed to write to stdin of a process
        StdinWrite(String),
        /// Failed to write to stdout
        StdoutWrite(String),
        /// Process hanged/can't close
        ProcessWait(String),
        /// Something wrong happened during a request
        Request(String),
        /// Unknown
        Other(String),
    }

    impl std::fmt::Display for IoError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Self::Other(err_string) => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("IoError").red().bold()
                    );
                    write!(
                        f,
                        r#"
CAUSE:
    {}
                    "#,
                        style(err_string).yellow()
                    )
                }
                Self::Request(err_string) => {
                    println!(
                        "{}{} {}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("IoError").red().bold(),
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("Request").red().bold()
                    );
                    write!(
                        f,
                        r#"
CAUSE:
    {}
                    "#,
                        style(err_string).yellow()
                    )
                }
                Self::StdoutWrite(err_string) => {
                    println!(
                        "{}{} {}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("IoError").red().bold(),
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("StdoutWrite").red().bold()
                    );
                    write!(
                        f,
                        r#"
CAUSE:
    failed to write to stdout.
    {}
                    "#,
                        style(err_string).yellow()
                    )
                }
                Self::ProcessWait(err_string)
                | Self::StdinWrite(err_string)
                | Self::ProcessSpawn(err_string) => {
                    println!(
                        "{}{} {}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("IoError").red().bold(),
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("Process").red().bold()
                    );
                    write!(
                        f,
                        r#"
CAUSE:
    {}
                    "#,
                        style(err_string).yellow()
                    )
                }
            }
        }
    }

    impl From<std::io::Error> for Error {
        fn from(err: std::io::Error) -> Self {
            Self::IO(IoError::Other(err.to_string()))
        }
    }

    impl From<reqwest::Error> for Error {
        fn from(err: reqwest::Error) -> Self {
            Self::IO(IoError::Request(err.to_string()))
        }
    }
}

/// Fetch module errors
pub mod fetch {
    use super::{style, Emoji, Error};

    #[derive(Clone)]
    #[non_exhaustive]
    pub enum FetchError {
        /// Unable to get secret right after couple tries
        ExaustedSecretRetries,
        /// Gistit hash doesn't exist in location
        NotFound,
        /// Unexpected Response
        UnexpectedResponse,
    }

    impl From<FetchError> for Error {
        fn from(err: FetchError) -> Self {
            Self::Fetch(err)
        }
    }

    impl std::fmt::Display for FetchError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Self::ExaustedSecretRetries => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("ExaustedSecretRetries").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    provided secret was incorrect too many times.
                        "#,
                        style("--secret <secret>").red().bold(),
                    )
                }
                Self::NotFound => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("NotFound").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}
    {}

CAUSE:
    gistit hash could not be found.
    it's lifespan might have expired or it's no longer being hosted.
                        "#,
                        style("--hash <hash>").red().bold(),
                        style("--url <url>").red().bold(),
                    )
                }
                Self::UnexpectedResponse => {
                    println!(
                        "{}{}",
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("UnexpectedResponse").red().bold()
                    );
                    write!(
                        f,
                        r#"
CAUSE:
    fetch destination returned an unexpected response.

This is unlikely to be caused by a misuse of the application.
The host location is missbehaving or trying to be evil.
                        "#
                    )
                }
            }
        }
    }
}

/// Clipboard module errors
pub mod clipboard {
    use super::{io, style, Emoji, Error};

    #[derive(Clone)]
    pub enum ClipboardError {
        /// Unsupported platform. android, ios...
        UnknownPlatform,
        /// Program binaries 'xclip' and 'xsel' not present
        MissingX11ClipboardBin,
        /// Program binary 'wl-copy' not present
        MissingWaylandClipboardBin,
        /// Program binary 'xauth' not present
        MissingTtyClipboardBin,
        /// Environment variable 'DISPLAY' is not set
        MissingDisplayEnvSsh,
        /// Program binary 'pbcopy' not present
        #[cfg(all(target_os = "macos", target_os = "ios"))]
        MissingMacosClipboardBin,
        /// Program binary crashed during execution
        BinExecution(io::IoError),
    }

    impl From<ClipboardError> for Error {
        fn from(err: ClipboardError) -> Self {
            Self::Clipboard(err)
        }
    }

    impl std::fmt::Display for ClipboardError {
        #[allow(clippy::too_many_lines)]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Self::UnknownPlatform => {
                    println!(
                        "{}{} {}{}",
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("Clipboard").yellow().bold(),
                        style(Emoji("\u{274c} ", "X")).red(),
                        style("UnknownPlatform").red().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    could not enable clipboard feature under this platform.
    supported platforms: Linux, BSD, Windows, MacOs.
                    "#,
                        style("--clipboard").red().bold()
                    )
                }
                Self::MissingX11ClipboardBin => {
                    println!(
                        "{}{} {}{}",
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("Clipboard").yellow().bold(),
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("MissingX11ClipboardBin").yellow().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    could not find any X11 clipboard program binaries. ('xclip', 'xsel')
    please consider installing of the above programs to have more reliable results.

This is not a fatal error, application will attempt the fallback OSC52 clipboard escape sequence.
                    "#,
                        style("--clipboard").yellow().bold()
                    )
                }
                Self::MissingWaylandClipboardBin => {
                    println!(
                        "{}{} {}{}",
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("Clipboard").yellow().bold(),
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("MissingWaylandClipboardBin").yellow().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    could not find Wayland clipboard program binary. ('wl-copy').
    please consider installing 'wl-clipboard' on your system to have more reliable results.

This is not a fatal error, application will attempt the fallback OSC52 clipboard escape sequence.
                    "#,
                        style("--clipboard").yellow().bold()
                    )
                }
                Self::MissingTtyClipboardBin => {
                    println!(
                        "{}{} {}{}",
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("Clipboard").yellow().bold(),
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("MissingTtyClipboardBin").yellow().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    could not find the installation for 'xauth' program.
    this likely means that display passthrough under SSH is not working properly.

This is not a fatal error, application will attempt the fallback OSC52 clipboard escape sequence.
                    "#,
                        style("--clipboard").yellow().bold()
                    )
                }
                Self::MissingDisplayEnvSsh => {
                    println!(
                        "{}{} {}{}",
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("Clipboard").yellow().bold(),
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("MissingDisplayEnvSsh").yellow().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    the environment variable 'DISPLAY' is not set.
    this likely means that display passthrough under SSH is not working properly.

This is not a fatal error, application will attempt the fallback OSC52 clipboard escape sequence.
                    "#,
                        style("--clipboard").yellow().bold()
                    )
                }
                Self::BinExecution(io_err) => match io_err {
                    io::IoError::ProcessSpawn(output) => {
                        println!(
                            "{}{} {}{}",
                            style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                            style("Clipboard").yellow().bold(),
                            style(Emoji("\u{274c} ", "X")).red(),
                            style("BinExecution").red().bold()
                        );
                        write!(
                            f,
                            r#"
PARAM:
    {}

CAUSE:
    could not spawn the clipboard program process.
    {}
                            "#,
                            style("--clipboard").red().bold(),
                            style(output).yellow()
                        )
                    }
                    io::IoError::StdinWrite(output) => {
                        println!(
                            "{}{} {}{}",
                            style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                            style("Clipboard").blue().bold(),
                            style(Emoji("\u{274c} ", "X")).red(),
                            style("BinExecution").red().bold()
                        );
                        write!(
                            f,
                            r#"
PARAM:
    {}

CAUSE:
    could not write to the stdin of the matched clipboard program process.
    {} 
                            "#,
                            style("--clipboard").red().bold(),
                            style(output).yellow()
                        )
                    }
                    io::IoError::ProcessWait(output)
                    | io::IoError::Other(output)
                    | io::IoError::StdoutWrite(output)
                    | io::IoError::Request(output) => {
                        println!(
                            "{}{} {}{}",
                            style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                            style("Clipboard").yellow().bold(),
                            style(Emoji("\u{274c} ", "X")).red(),
                            style("BinExecution").red().bold()
                        );
                        write!(
                            f,
                            r#"
PARAM:
    {}

CAUSE:
    the clipboard program process crashed:
    {}
                            "#,
                            style("--clipboard").red().bold(),
                            style(output).yellow()
                        )
                    }
                },
                #[cfg(all(target_os = "macos", target_os = "ios"))]
                Self::MissingMacosClipboardBin => {
                    println!(
                        "{}{} {}{}",
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("Clipboard").yellow().bold(),
                        style(Emoji("\u{26a0}\u{fe0f}  ", "!")).yellow(),
                        style("MissingMacosClipboardBin").yellow().bold()
                    );
                    write!(
                        f,
                        r#"
PARAM:
    {}

CAUSE:
    could not find Macos clipboard program binary. ('pbcopy').
    please consider installing 'pbcopy' on your system to have more reliable results.

This is not a fatal error, application will attempt the fallback OSC52 clipboard escape sequence.
                    "#,
                        style("--clipboard").yellow().bold(),
                    )
                }
            }
        }
    }
}
