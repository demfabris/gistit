//! Clipboard module
//!
//! The clipboard feature in Gistit is nothing more than a quality of life bonus to automatically store the
//! Gistit hash into your system clipboard. Since we're interested in persisting the Gistit hash
//! after the program exists we have to rely on not so reliable methods to achieve this behaviour.
//!
//! Here we do our best efforts look for the most common clipboard binaries, spawn a child process, and pipe the
//! contents into it's 'stdin'. If no binary was found we'll fallback to OSC52 escape sequence.
//! [OSC52](https://www.reddit.com/r/vim/comments/k1ydpn/a_guide_on_how_to_copy_text_from_anywhere/)
//!
//! credits: this implementation is heavily inspired on
//! [copypasta](https://docs.rs/copypasta/0.7.1/copypasta/)
//!
//! **note** we're not interested in 'paste' functionallity
//!
//! # Linux/BSD
//!
//! On Linux/BSD we'll match the display server and attempt to find related
//! clipboard binaries.
//!
//! ## WSL
//!
//! Will use `clip.exe` to pipe content into.
//!
//! ## X11
//!
//! Will look for `xclip`, `xsel` and use it in this order of preference.
//!
//! ## Wayland
//!
//! Will look for `wl-copy` binary.
//!
//! ## Tty (SSH session)
//!
//! Under this condition we'll do a couple of extra checks to ensure X11 Passthrough is
//! working, otherwise clipboard usage is unlikely to succeed (?).
//!
//! 1. checks for `xauth` binary, utility to manage X11 session cookies.
//! 2. reads `DISPLAY` env variable to ensure it's set with 'localhost:' something something.
//!
//! If the above are ok we check for X11 clipboard binaries to use.
//!
//! # Mac OS
//!
//! We check for `pbcopy` binary but it's absence is not a showstopper since we can still try
//! OSC52 escape sequence.
//!
//! # Windows
//!
//! Doesn't make sense to check for `clip.exe` because it's default installation. Anyhow, we're
//! not using it under this platform. This can change in the future
use std::env;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use which::which;

use crate::{error, Result};

/// The clipboard structure, holds the content string
#[derive(Clone, Debug)]
pub struct Clipboard {
    content: String,
}

/// The clipboard with the display server figured out
#[derive(Clone, Debug)]
pub struct Selected {
    display: DisplayKind,
    content: String,
}

/// The clipboard that attempts the external binary approach
#[derive(Clone, Debug)]
pub struct Binary {
    bin: OsString,
    selected: Selected,
    program: ClipboardBinProgram,
}

/// The clipboard that attempts OSC52 escape sequence approach
#[derive(Clone, Debug)]
pub struct EscapeSequence {
    selected: Selected,
}

/// The display server type
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
enum DisplayKind {
    X11,
    Wayland,
    Wsl,
    SshTty,
    Unknown,
    #[cfg(target_is = "macos")]
    MacOs,
    #[cfg(target_os = "windows")]
    Windows,
}

/// Returns the current display server
fn select_display() -> DisplayKind {
    #[cfg(target_os = "windows")]
    return DisplayKind::Windows;

    #[cfg(target_is = "macos")]
    return DisplayKind::MacOs;

    // Linux/BSD only
    if is_wsl() {
        DisplayKind::Wsl
    } else if is_wayland() {
        DisplayKind::Wayland
    } else if is_x11() {
        DisplayKind::X11
    } else if is_ssh_tty() {
        DisplayKind::SshTty
    } else {
        DisplayKind::Unknown
    }
}

/// Checks whether we're under windows subsystem for linux
#[cfg(target_family = "unix")]
fn is_wsl() -> bool {
    env::var("WSL_DISTRO_NAME").is_ok()
        || env::var("WT_SESSION").is_ok()
        || env::var("WSL_INTEROP").is_ok()
}

/// Check whether or not in Wayland environment
/// This function is avaiable only under Linux/BSD environment so no extra checks are needed.
/// **note** that this is best to run before checking for X11 because `DISPLAY` var can also be set
/// under Wayland.
#[cfg(all(
    target_family = "unix",
    not(all(target_os = "macos", target_os = "ios", target_os = "android"))
))]
fn is_wayland() -> bool {
    let mut score = 0;
    match env::var("XDG_SESSION_TYPE").ok().as_deref() {
        Some("wayland") => score += 1,
        Some(_) | None => (),
    }
    if env::var("WAYLAND_DISPLAY").is_ok() {
        score += 1;
    }
    score > 0
}

/// Check whether or not in X11
/// This function is avaiable only under Linux/BSD environment so no extra checks are needed.
#[cfg(all(
    target_family = "unix",
    not(all(target_os = "macos", target_os = "ios", target_os = "android"))
))]
fn is_x11() -> bool {
    let mut score = 0;
    match env::var("XDG_SESSION_TYPE").ok().as_deref() {
        Some("x11") => score += 1,
        Some(_) | None => (),
    }
    if env::var("DISPLAY").is_ok() {
        score += 1;
    }
    score > 0
}

/// Checks whether or not in TTY.
/// The default session type under SSH is `tty` so we make sure to assert both things
/// since we're not supporting clipboard under raw tty sessions.
#[cfg(all(
    target_family = "unix",
    not(all(target_os = "macos", target_os = "ios", target_os = "android"))
))]
fn is_ssh_tty() -> bool {
    let tty = env::var("XDG_SESSION_TYPE").as_deref() == Ok("tty");
    let ssh = env::var("SSH_CLIENT").is_ok();
    tty && ssh
}

impl Clipboard {
    /// Creates a new Clipboard instance with the content string
    #[must_use]
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_owned(),
        }
    }

    /// Tries to select the current display
    ///
    /// # Errors
    ///
    /// Fails with [`ClipboardError`] error
    pub fn try_into_selected(self) -> Result<Selected> {
        match select_display() {
            DisplayKind::Unknown => Err(error::Clipboard::UnsupportedPlatform.into()),
            valid => Ok(Selected {
                display: valid,
                content: self.content,
            }),
        }
    }
}

/// The trait that a ready-to-use clipboard implements
pub trait Provider {
    /// Attempt to set the contents into the system clipboard
    ///
    /// # Errors
    ///
    /// Fails with [`ClipboardError`]
    fn set_contents(&self) -> Result<()>;
}

impl Provider for Binary {
    fn set_contents(&self) -> Result<()> {
        let mut command = Command::new(&self.bin);
        match self.program {
            ClipboardBinProgram::Xclip => {
                command.arg("-sel").arg("clip");
            }
            ClipboardBinProgram::Xsel => {
                command.arg("--clipboard");
            }
            ClipboardBinProgram::WlCopy | ClipboardBinProgram::ClipExe => (),
        };
        let mut process = command
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        process
            .stdin
            .as_mut()
            .expect("to access stdin")
            .write_all(self.selected.content.as_bytes())?;

        let _status = process.wait()?;

        Ok(())
    }
}

impl Provider for EscapeSequence {
    fn set_contents(&self) -> Result<()> {
        print!("\x1B]52;c;{}\x07", base64::encode(&self.selected.content));
        Ok(())
    }
}

impl Selected {
    /// Transforms this clipboard into a ready-to-use kind
    /// First checks for binaries and fallbacks to the ANSI escape sequence approach.
    #[must_use]
    pub fn into_provider(self) -> Box<dyn Provider> {
        match self.try_into_bin() {
            Ok(bin_clipboard) => {
                return Box::new(bin_clipboard);
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
        Box::new(EscapeSequence { selected: self })
    }
}

/// Currently supported clipboard programs
#[non_exhaustive]
#[derive(Clone, Debug)]
enum ClipboardBinProgram {
    Xclip,
    Xsel,
    ClipExe,
    WlCopy,
    #[cfg(all(target_os = "macos", target_os = "ios"))]
    PbCopy,
}

#[cfg(all(
    target_family = "unix",
    not(all(target_os = "macos", target_os = "ios", target_os = "android"))
))]
impl Selected {
    /// Checks for supported clipboard binaries and attempts to convert the selected clipboard into
    /// the binary implementation.
    ///
    /// # Errors
    ///
    /// Will fail with [`ClipboardError`] when any matched display server misses it's supported
    /// clipboard binaries.
    fn try_into_bin(&self) -> Result<Binary> {
        let (bin, program) = match self.display {
            DisplayKind::X11 => {
                let mut binaries = [
                    (which("xclip"), ClipboardBinProgram::Xclip),
                    (which("xsel"), ClipboardBinProgram::Xsel),
                    // TODO: Add more supported clipboard programs here
                ]
                .into_iter();

                let (bin, program) = binaries
                    .find(|(bin, _)| bin.is_ok())
                    .ok_or(error::Clipboard::MissingBinary)?;
                // Safe to unwrap since we previously checked `bin.is_ok()`
                (bin.unwrap(), program)
            }
            DisplayKind::Wayland => {
                let bin = which("wl-copy")?;
                let program = ClipboardBinProgram::WlCopy;
                (bin, program)
            }
            DisplayKind::SshTty => {
                //`xauth` missing most likely mean display passthrough isn't working
                let _xauth = which("xauth")?;

                // DISPALY variable different than `localhost:...` is a bad sign as well
                env::var("DISPLAY").map_err(|_| error::Clipboard::DisplayNotSet)?;

                let mut binaries = [
                    (which("xclip"), ClipboardBinProgram::Xclip),
                    (which("xsel"), ClipboardBinProgram::Xsel),
                    // TODO: Add more supported clipboard programs here
                ]
                .into_iter();

                let (bin, program) = binaries
                    .find(|(bin, _)| bin.is_ok())
                    .ok_or(error::Clipboard::MissingBinary)?;
                // Safe to unwrap since we previously checked `bin.is_ok()`
                (bin.unwrap(), program)
            }
            DisplayKind::Wsl => {
                let bin = PathBuf::from("clip.exe");
                let program = ClipboardBinProgram::ClipExe;
                (bin, program)
            }
            DisplayKind::Unknown => panic!("clipboard feature not supported"),
        };
        Ok(Binary {
            bin: bin.as_os_str().to_owned(),
            selected: self.clone(),
            program,
        })
    }
}

#[cfg(all(target_os = "macos", target_os = "ios"))]
impl Selected {
    /// Checks for supported clipboard binaries and attempts to convert the selected clipboard into
    /// the binary implementation.
    ///
    /// # Errors
    ///
    /// Will fail with [`ClipboardError`] when any matched display server misses it's supported
    /// clipboard binaries.
    fn try_into_bin(&self) -> Result<Binary> {
        let bin = match self.display {
            DisplayKind::MacOs => which("pbcopy")
                .ok()
                .map(|t| t.as_os_str().to_owned())
                .ok_or(ErrorKind::MissingClipboardBinary)?,
            DisplayKind::Unknown => panic!("clipboard feature not supported"),
        };
        let program = ClipboardBinProgram::PbCopy;
        Ok(Binary {
            bin,
            program,
            selected: self.clone(),
        })
    }
}

/// Not supported
#[cfg(target_os = "windows")]
impl Selected {
    fn try_into_bin(&self) -> Result<Binary> {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    pub fn clipboard_test_selection_order() {
        env::remove_var("DISPLAY");
        env::remove_var("WSL_DISTRO_NAME");
        env::remove_var("WAYLAND_DISPLAY");
        env::remove_var("SSH_CLIENT");
        env::remove_var("WT_SESSION");
        env::remove_var("WSL_INTEROP");

        env::set_var("DISPLAY", "localhost");
        let clip1 = Clipboard::new("foo").try_into_selected().unwrap();
        assert_eq!(clip1.display, DisplayKind::X11);

        env::set_var("WAYLAND_DISPLAY", "wayland");
        let clip2 = Clipboard::new("bar").try_into_selected().unwrap();
        assert_eq!(clip2.display, DisplayKind::Wayland);

        env::set_var("WSL_DISTRO_NAME", "hanna_montana_linux");
        let clip3 = Clipboard::new("baz").try_into_selected().unwrap();
        assert_eq!(clip3.display, DisplayKind::Wsl);
    }
}
