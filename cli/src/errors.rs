//! Errors module
//!
//! Here we define all errors (fatal and non-fatal) that gistit can output
//! We moved to manual implementation of the errors to allow for prettier and colorized output
//! Every error should convert to top level [`Error`] in the end.
use console::style;

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
            Self::Argument => {
                write!(f, "Something went wrong during arg parsing")
            }
        }
    }
}

/// Encryption module errors
pub mod encryption {
    use super::{style, Error};
    use crypto::symmetriccipher::SymmetricCipherError;

    pub enum EncryptionError {
        SecretLength,
        /// Errors related to ciphering. Output from 'rust-crypto' crate symmetric cipher ops
        CipherError(SymmetricCipherError),
    }

    impl From<SymmetricCipherError> for EncryptionError {
        fn from(err: SymmetricCipherError) -> Self {
            Self::CipherError(err)
        }
    }

    impl From<EncryptionError> for Error {
        fn from(err: EncryptionError) -> Self {
            Self::Encryption(err)
        }
    }

    impl std::fmt::Display for EncryptionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                EncryptionError::SecretLength => {
                    println!("{}", style("\u{274c} SecretLength").red().bold());
                    write!(
                        f,
                        r#"
Invalid secret character length.
min = {} max = {}
                    "#,
                        style("5 chars").yellow(),
                        style("50 chars").yellow()
                    )
                }
                EncryptionError::CipherError(err) => {
                    println!("{}", style("\u{274c} CipherError").red().bold());
                    write!(
                        f,
                        r#"
The encryption process failed:

{:?}
                    "#,
                        err
                    )
                }
            }
        }
    }
}

/// Params module errors
pub mod params {
    use super::{style, Error};

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
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                ParamsError::DescriptionCharRange => {
                    println!("{}", style("\u{274c} DescriptionCharLength").red().bold());
                    write!(
                        f,
                        r#"
Invalid description character length.
min = {} max = {}
                    "#,
                        style("10 chars").yellow(),
                        style("100 chars").yellow()
                    )
                }
                ParamsError::AuthorCharRange => {
                    println!("{}", style("\u{274c} AuthorCharRange").red().bold());
                    write!(
                        f,
                        r#"
Invalid author character length.
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
                    println!("{}", style("\u{274c} Colorscheme").red().bold());
                    write!(
                        f,
                        r#"
Invalid colorscheme parameter.
Run '{}' to list supported colorschemes.{}
                    "#,
                        style("gistit --colorschemes").green(),
                        suggest.unwrap_or_else(|| "".to_string())
                    )
                }
                ParamsError::LifespanRange => {
                    println!("{}", style("\u{274c} LifespanRange").red().bold());
                    write!(
                        f,
                        r#"
Invalid lifespan parameter value range.
min = {} max = {}
                    "#,
                        style("300s").yellow(),
                        style("3600s (default)").yellow()
                    )
                }
                ParamsError::InvalidLifespan => {
                    println!("{}", style("\u{274c} InvalidLifespan").red().bold());
                    write!(
                        f,
                        r#"
Invalid lifespan parameter.
Input is not a positive number
                    "#,
                    )
                }
                ParamsError::InvalidUrl(err) => {
                    println!("{}", style("\u{274c} InvalidUrl").red().bold());
                    write!(
                        f,
                        r#"
Input is not a valid URL:

{}
                    "#,
                        style(err).yellow()
                    )
                }
                ParamsError::InvalidHash(hash_captured) => {
                    println!("{}", style("\u{274c} InvalidHash").red().bold());
                    write!(
                        f,
                        r#"
Input is not a valid gistit hash

got: {}
                    "#,
                        style(hash_captured).yellow()
                    )
                }
            }
        }
    }
}

/// File module errors
pub mod file {
    use super::{style, Error};

    #[derive(Clone)]
    pub enum FileError {
        /// File extension doesn't match supported ones
        UnsupportedExtension(String),
        /// File size is outside allowed range
        UnsupportedSize(u64),
        /// File is not a file
        UnsupportedType(String),
        /// File has weird extensions
        MissingExtension,
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
                    println!("{}", style("\u{274c} MissingFileExtension").red().bold());
                    write!(
                        f,
                        r#"
Input file must have an extension.
see supported file extensions here: {}
                    "#,
                        style("https://rust-lang.org").blue()
                    )
                }
                Self::UnsupportedExtension(ext) => {
                    println!("{}", style("\u{274c} UnsupportedExtension").red().bold());
                    write!(
                        f,
                        r#"
File extension not currently supported: '{}'
see supported file extensions here: {}
                    "#,
                        style(ext).cyan(),
                        style("https://rust-lang.org").blue()
                    )
                }
                Self::UnsupportedType(name) => {
                    println!("{}", style("\u{274c} UnsupportedType").red().bold());
                    write!(f, "Input '{}' is not a file", style(name).cyan())
                }
                Self::UnsupportedSize(size) => {
                    println!("{}", style("\u{274c} UnsupportedSize").red().bold());
                    let size_str = if size > &1 {
                        format!("{} bytes", size.to_string())
                    } else {
                        format!("{} byte", size.to_string())
                    };
                    write!(
                        f,
                        r#"
File size is not in allowed range. ({})
min = {} max = {}
                    "#,
                        style(size_str).red().bold(),
                        style("20 bytes").yellow(),
                        style("200 kb").yellow()
                    )
                }
            }
        }
    }
}

/// I/O operations error
pub mod io {
    use super::{style, Error};

    #[derive(Clone)]
    pub enum IoError {
        /// Failed to spawn a process
        ProcessSpawn(String),
        /// Failed to write to stdin of a process
        StdinWrite(String),
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
                Self::Other(err_string) | Self::Request(err_string) => {
                    println!("{}", style("\u{274c} IoError").red().bold());
                    write!(f, "\n{}", style(err_string).yellow())
                }
                Self::ProcessWait(err_string)
                | Self::StdinWrite(err_string)
                | Self::ProcessSpawn(err_string) => {
                    println!(
                        "{} {}",
                        style("\u{274c} IoError").red().bold(),
                        style("\u{274c} Process").red().bold()
                    );
                    write!(f, "\n{}", style(err_string).yellow())
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

/// Clipboard module errors
pub mod clipboard {
    use super::{io, style, Error};

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
                        "{} {}",
                        style("Clipboard").blue().bold(),
                        style("\u{274c} UnknownPlatform").red().bold()
                    );
                    write!(
                        f,
                        r#"
Could not enable clipboard feature under this platform.
supported platforms: Linux, BSD, Windows, MacOs.
                    "#
                    )
                }
                Self::MissingX11ClipboardBin => {
                    println!(
                        "{} {}",
                        style("\u{26a0}\u{fe0f} Clipboard").blue().bold(),
                        style("MissingX11ClipboardBin").yellow().bold()
                    );
                    write!(
                        f,
                        r#"
Could not find any X11 clipboard program binaries. ('xclip', 'xsel')
Please consider installing of the above programs to have more reliable results.

This is not a fatal error, application will attempt the fallback OSC52 clipboard escape sequence.
                    "#,
                    )
                }
                Self::MissingWaylandClipboardBin => {
                    println!(
                        "{} {}",
                        style("\u{26a0}\u{fe0f} Clipboard").blue().bold(),
                        style("MissingWaylandClipboardBin").yellow().bold()
                    );
                    write!(
                        f,
                        r#"
Could not find Wayland clipboard program binary. ('wl-copy').
Please consider installing 'wl-clipboard' on your system to have more reliable results.

This is not a fatal error, application will attempt the fallback OSC52 clipboard escape sequence.
                    "#,
                    )
                }
                Self::MissingTtyClipboardBin => {
                    println!(
                        "{} {}",
                        style("\u{26a0}\u{fe0f} Clipboard").blue().bold(),
                        style("MissingTtyClipboardBin").yellow().bold()
                    );
                    write!(
                        f,
                        r#"
Could not find the installation for 'xauth' program.
This likely means that display passthrough under SSH is not working properly.

This is not a fatal error, application will attempt the fallback OSC52 clipboard escape sequence.
                    "#,
                    )
                }
                Self::MissingDisplayEnvSsh => {
                    println!(
                        "{} {}",
                        style("\u{26a0}\u{fe0f} Clipboard").blue().bold(),
                        style("MissingDisplayEnvSsh").yellow().bold()
                    );
                    write!(
                        f,
                        r#"
The environment variable 'DISPLAY' is not set.
This likely means that display passthrough under SSH is not working properly.

This is not a fatal error, application will attempt the fallback OSC52 clipboard escape sequence.
                    "#,
                    )
                }
                Self::BinExecution(io_err) => match io_err {
                    io::IoError::ProcessSpawn(output) => {
                        println!(
                            "{} {}",
                            style("Clipboard").blue().bold(),
                            style("\u{274c} BinExecution").red().bold()
                        );
                        write!(
                            f,
                            r#"
Could not spawn the clipboard program process.
This is not expected, check if you have permission to execute programs.

{}
                            "#,
                            output
                        )
                    }
                    io::IoError::StdinWrite(output) => {
                        println!(
                            "{} {}",
                            style("Clipboard").blue().bold(),
                            style("\u{274c} BinExecution").red().bold()
                        );
                        write!(
                            f,
                            r#"
Could not write to the stdin of the matched clipboard program process.
This is not expected, check if your clipboard program installation is healthy.

{} 
                            "#,
                            output
                        )
                    }
                    io::IoError::ProcessWait(output)
                    | io::IoError::Other(output)
                    | io::IoError::Request(output) => {
                        println!(
                            "{} {}",
                            style("Clipboard").blue().bold(),
                            style("\u{274c} BinExecution").red().bold()
                        );
                        write!(
                            f,
                            r#"
The clipboard program process crashed:

{}
                            "#,
                            output
                        )
                    }
                },
                #[cfg(all(target_os = "macos", target_os = "ios"))]
                Self::MissingMacosClipboardBin => {
                    println!(
                        "{} {}",
                        style("\u{26a0}\u{fe0f} Clipboard").blue().bold(),
                        style("MissingMacosClipboardBin").yellow().bold()
                    );
                    write!(
                        f,
                        r#"
Could not find Macos clipboard program binary. ('pbcopy').
Please consider installing 'pbcopy' on your system to have more reliable results.

This is not a fatal error, application will attempt the fallback OSC52 clipboard escape sequence.
                    "#,
                    )
                }
            }
        }
    }

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
}
