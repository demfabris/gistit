//! The host module
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
use std::ffi::OsStr;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use unchecked_unwrap::UncheckedUnwrap;

use async_trait::async_trait;
use clap::ArgMatches;
use console::Style;
use directories::BaseDirs;

use lib_gistit::encrypt::{HashedSecret, Secret};
use lib_gistit::errors::internal::InternalError;
use lib_gistit::errors::io::IoError;
use lib_gistit::file::{File, FileReady};
use lib_gistit::{Error, Result};

use crate::dispatch::Dispatch;
use crate::gistit_line_out;
use crate::params::{HostParams, Params};

const UNIX_SIGKILL: i32 = 9;
const UNIX_SIGNOTHING: i32 = 0;

/// The Send action runtime parameters
#[derive(Debug, Clone)]
// No way around it, it's arg parsing
#[allow(clippy::struct_excessive_bools)]
pub struct Action {
    /// Start p2p background process
    pub start: bool,
    /// Auto copy to clipboard
    pub clipboard: bool,
    /// The seed to derive the keypair
    pub seed: Option<&'static str>,
    /// Wether or not to save peers
    pub persist: bool,
    /// Address to listen for connections
    pub listen: &'static str,
    /// Stop p2p background process
    pub stop: bool,
    /// Display background process status
    pub status: bool,
    /// The secret key to protect your gistits.
    pub secret: Option<&'static str>,
    /// The file to be hosted.
    pub file: Option<&'static OsStr>,
    /// The private network to join
    pub join: Option<&'static str>,
}

impl Action {
    /// Parse [`ArgMatches`] into the dispatchable Host action.
    /// Here we also merge user settings while keeping this order of priority:
    /// arguments > local settings file > app defaults
    ///
    /// # Errors
    ///
    /// Fails with argument errors
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        // merge settings

        Ok(Box::new(Self {
            start: args.is_present("start"),
            clipboard: args.is_present("clipboard"),
            seed: args.value_of("seed"),
            persist: args.is_present("persist"),
            listen: args.value_of("listen").expect("to have default value"),
            stop: args.is_present("stop"),
            status: args.is_present("status"),
            secret: args.value_of("secret"),
            file: args.value_of_os("file"),
            join: args.value_of("join"),
        }))
    }
}

pub enum ProcessCommand {
    StartWithSeed(&'static str),
    Start,
    Stop,
    Status,
    Skip,
}

pub struct Config {
    process_command: ProcessCommand,
    maybe_secret: Option<HashedSecret>,
    maybe_file: Option<Box<dyn FileReady + Send + Sync>>,
}

impl Config {
    #[must_use]
    pub fn new(
        process_command: ProcessCommand,
        maybe_secret: Option<HashedSecret>,
        maybe_file: Option<Box<dyn FileReady + Send + Sync>>,
    ) -> Self {
        Self {
            process_command,
            maybe_secret,
            maybe_file,
        }
    }
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&'static self) -> Result<Self::InnerData> {
        let params = Params::from_host(self).check_consume()?;
        let command = match (self.start, self.seed, self.stop, self.status) {
            (true, Some(seed), false, false) => ProcessCommand::StartWithSeed(seed),
            (true, None, false, false) => ProcessCommand::Start,
            (false, None, true, false) => ProcessCommand::Stop,
            (false, None, false, true) => ProcessCommand::Status,
            (_, _, _, _) => ProcessCommand::Skip,
        };

        // Construct `FileReady` if a file was provided, that means user wants to host a new file
        // and the background process should be running.
        let (maybe_file, maybe_hashed_secret): (
            Option<Box<dyn FileReady + Send + Sync>>,
            Option<HashedSecret>,
        ) = if let Some(file) = self.file {
            let path = Path::new(file);
            let file = File::from_path(path).await?.check_consume().await?;

            // If secret provided, hash it and encrypt file
            if let Some(secret_str) = self.secret {
                let hashed_secret = Secret::new(secret_str).check_consume()?.into_hashed()?;
                gistit_line_out!("Encrypting...");

                let encrypted_file = file.into_encrypted(secret_str).await?;
                (Some(Box::new(encrypted_file)), Some(hashed_secret))
            } else {
                (Some(Box::new(file)), None)
            }
        } else {
            (None, None)
        };

        Ok(Config::new(command, maybe_hashed_secret, maybe_file))
    }

    async fn dispatch(&'static self, config: Self::InnerData) -> Result<()> {
        let runtime_dir = get_runtime_dir()?;
        let cache_dir = runtime_dir.join("gistit_peers");
        let mut node_hash: &str;
        // SAFETY: Value was previously checked in `Dispatch::prepare` stage. It's either the
        // default '127.0.0.1:0' or some valid <ip>:<port>
        let (addr, port) = unsafe { self.listen.split_once(':').unchecked_unwrap() };

        if !Path::exists(&cache_dir) {
            std::fs::create_dir(&cache_dir)?;
        }

        match config.process_command {
            ProcessCommand::StartWithSeed(password) => {
                gistit_line_out!("Starting gistit network node process with seed...");
            }
            ProcessCommand::Start => {
                gistit_line_out!("Starting gistit network node process...");
            }
            ProcessCommand::Stop => {
                gistit_line_out!("Stopping gistit network node process...");
            }
            ProcessCommand::Status => {
                todo!()
            }
            // Not a process instruction
            ProcessCommand::Skip => {
                if let Self::InnerData {
                    maybe_file: Some(file),
                    ..
                } = config
                {
                    todo!()
                }
            }
        };
        Ok(())
    }
}

#[cfg(target_family = "unix")]
fn get_runtime_dir() -> Result<PathBuf> {
    let dirs = BaseDirs::new().ok_or_else(|| {
        Error::Internal(InternalError::Other("no valid home directory".to_owned()))
    })?;
    Ok(dirs
        .runtime_dir()
        .unwrap_or_else(|| Path::new("/tmp"))
        .to_path_buf())
}
