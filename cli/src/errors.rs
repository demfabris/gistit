//! Errors module
//!
//! Here we define all errors (fatal and non-fatal) that gistit can output
//! We moved to manual implementation of the errors to allow for prettier and colorized output
//! Every error should convert to top level [`Error`] in the end.
use colored::Colorize;

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
    use super::{Colorize, Error};
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
                    println!("{}", "[SecretLength]".red());
                    write!(
                        f,
                        r#"
invalid description character length.
MIN = {} MAX = {}
                    "#,
                        "5 chars".yellow(),
                        "50 chars".yellow()
                    )
                }
                EncryptionError::CipherError(err) => {
                    println!("{}", "[CipherError]".red());
                    write!(
                        f,
                        r#"
Something went wrong during encryption.
{}: {:?}
                    "#,
                        "Reason".bright_magenta(),
                        err
                    )
                }
            }
        }
    }
}

/// Params module errors
pub mod params {
    use super::{Colorize, Error};

    pub enum ParamsError {
        DescriptionCharRange,
        AuthorCharRange,
        Colorscheme(Option<String>),
        LifespanRange,
        InvalidLifespan,
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
                    println!("{}", "[DescriptionCharLength]".red());
                    write!(
                        f,
                        r#"
invalid description character length.
MIN = {} MAX = {}
                    "#,
                        "10 chars".yellow(),
                        "100 chars".yellow()
                    )
                }
                ParamsError::AuthorCharRange => {
                    println!("{}", "[AuthorCharRange]".red());
                    write!(
                        f,
                        r#"
invalid author character length.
MIN = {} MAX = {}
                    "#,
                        "3 chars".yellow(),
                        "30 chars".yellow()
                    )
                }
                ParamsError::Colorscheme(maybe_close_match) => {
                    let suggest = maybe_close_match.as_ref().map(|close_match| {
                        format!("\n\nDid you mean: '{}'?", close_match.bright_blue())
                    });
                    println!("{}", "[Colorscheme]".red());
                    write!(
                        f,
                        r#"
invalid colorscheme parameter.
run '{}' to list supported colorschemes.{}
                    "#,
                        "gistit --colorschemes".green(),
                        suggest.unwrap_or_else(|| "".to_string())
                    )
                }
                ParamsError::LifespanRange => {
                    println!("{}", "[LifespanRange]".red());
                    write!(
                        f,
                        r#"
invalid lifespan parameter.
MIN = {} MAX = {}
                    "#,
                        "300s".yellow(),
                        "3600s (default)".yellow()
                    )
                }
                ParamsError::InvalidLifespan => {
                    println!("{}", "[InvalidLifespan]".red());
                    write!(
                        f,
                        r#"
invalid lifespan parameter.
input is not a positive number
                    "#,
                    )
                }
            }
        }
    }
}

/// File module errors
pub mod file {
    use super::{Colorize, Error};

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
                    println!("{}", "[MissingFileExtension]".red());
                    write!(
                        f,
                        r#"
input file must have an extension.
see avaiable file extensions here: {}
                    "#,
                        "https://rust-lang.org".bright_blue()
                    )
                }
                Self::UnsupportedExtension(ext) => {
                    println!("{}", "[UnsupportedExtension]".red());
                    write!(
                        f,
                        r#"
file extension not currently supported: '{}'
see avaiable file extensions here: {}
                    "#,
                        ext.cyan(),
                        "https://rust-lang.org".bright_blue()
                    )
                }
                Self::UnsupportedType(name) => {
                    println!("{}", "[UnsupportedType]".red());
                    write!(
                        f,
                        r#"
input '{}' is not a file
                    "#,
                        name.cyan()
                    )
                }
                Self::UnsupportedSize(size) => {
                    println!("{}", "[UnsupportedSize]".red());
                    let size_str = if size > &1 {
                        format!("{} bytes", size.to_string())
                    } else {
                        format!("{} byte", size.to_string())
                    };
                    write!(
                        f,
                        r#"
file size is not in allowed range. ({})
MIN = {} MAX = {}
                    "#,
                        size_str.bright_red(),
                        "20 bytes".yellow(),
                        "200 kb".yellow()
                    )
                }
            }
        }
    }
}

/// I/O operations error
pub mod io {
    use super::{Colorize, Error};

    #[derive(Clone)]
    pub enum IoError {
        /// Failed to spawn a process
        ProcessSpawn(String),
        /// Failed to write to stdin of a process
        StdinWrite(String),
        /// Process hanged/can't close
        ProcessWait(String),
        Other(String),
    }

    impl std::fmt::Display for IoError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                Self::Other(err_string) => {
                    println!("{}", "[IoError]".red());
                    write!(
                        f,
                        r#"
Something went wrong during an I/O operation:
{}
                    "#,
                        err_string.yellow()
                    )
                }
                Self::ProcessWait(err_string)
                | Self::StdinWrite(err_string)
                | Self::ProcessSpawn(err_string) => {
                    println!("{} {}", "[IoError]".red(), "[Process]".red());
                    write!(
                        f,
                        r#"
Something went wrong during an I/O operation:
{}
                    "#,
                        err_string.yellow()
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
}

/// Clipboard module errors
pub mod clipboard {
    use super::{io, Colorize, Error};

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
                    println!("{} {}", "[Clipboard]".blue(), "[UnknownPlatform]".red());
                    write!(
                        f,
                        r#"
Could not enable clipboard feature under this platform
supported platforms: Linux, BSD, Windows, MacOs.
                    "#
                    )
                }
                Self::MissingX11ClipboardBin => {
                    println!(
                        "{} {}",
                        "[Clipboard]".blue(),
                        "[MissingX11ClipboardBin]".yellow()
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
                        "[Clipboard]".blue(),
                        "[MissingWaylandClipboardBin]".yellow()
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
                        "[Clipboard]".blue(),
                        "[MissingTtyClipboardBin]".yellow()
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
                        "[Clipboard]".blue(),
                        "[MissingDisplayEnvSsh]".yellow()
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
                        println!("{} {}", "[Clipboard]".blue(), "[BinExecution]".red());
                        write!(
                            f,
                            r#"
Could not spawn the clipboard program process.
This is not expected, check if you have permission to execute programs.

{}: {}
                            "#,
                            "Reason".bright_magenta(),
                            output
                        )
                    }
                    io::IoError::StdinWrite(output) => {
                        println!("{} {}", "[Clipboard]".blue(), "[BinExecution]".red());
                        write!(
                            f,
                            r#"
Could not write to the stdin of the matched clipboard program process.
This is not expected, check if your clipboard program installation is healthy.

{}: {} 
                            "#,
                            "Reason".bright_magenta(),
                            output
                        )
                    }
                    io::IoError::ProcessWait(output) | io::IoError::Other(output) => {
                        println!("{} {}", "[Clipboard]".blue(), "[BinExecution]".red());
                        write!(
                            f,
                            r#"
The clipboard program process crashed.
Something wen't wrong during execution

{}: {}
                            "#,
                            "Reason".bright_magenta(),
                            output
                        )
                    }
                },
                #[cfg(all(target_os = "macos", target_os = "ios"))]
                Self::MissingMacosClipboardBin => {
                    println!(
                        "{} {}",
                        "[Clipboard]".blue(),
                        "[MissingMacosClipboardBin]".yellow()
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
