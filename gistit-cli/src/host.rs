//! The host module
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use directories::BaseDirs;

use lib_gistit::encrypt::{HashedSecret, Secret};
use lib_gistit::file::{File, FileReady};
use lib_gistit::ipc::Bridge;

use crate::dispatch::Dispatch;
use crate::params::Params;
use crate::{prettyln, ErrorKind, Result};

#[derive(Debug, Clone)]
pub struct Action {
    pub start: Option<&'static str>,
    pub stop: bool,
    pub status: bool,
    pub clipboard: bool,
    pub host: &'static str,
    pub port: &'static str,
    pub file: Option<&'static OsStr>,
    pub secret: Option<&'static str>,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        // merge settings

        Ok(Box::new(Self {
            start: args.value_of("start"),
            clipboard: args.is_present("clipboard"),
            // SAFETY: Has default values
            host: unsafe { args.value_of("host").unwrap_unchecked() },
            port: unsafe { args.value_of("port").unwrap_unchecked() },
            stop: args.is_present("stop"),
            status: args.is_present("status"),
            secret: args.value_of("secret"),
            file: args.value_of_os("file"),
        }))
    }
}

pub enum ProcessCommand {
    Start(&'static str),
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
        Params::from_host(self).check_consume()?;
        let command = match (self.start, self.stop, self.status) {
            (Some(seed), false, false) => ProcessCommand::Start(seed),
            (None, true, false) => ProcessCommand::Stop,
            (None, false, true) => ProcessCommand::Status,
            (_, _, _) => ProcessCommand::Skip,
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
                prettyln!("Encrypting...");

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

        // let bridge = Bridge::connect(&runtime_dir)?;
        match config.process_command {
            ProcessCommand::Start(seed) => {
                let pid = spawn(&runtime_dir, seed, self.host, self.port)?;
                prettyln!(
                    "Starting gistit network node process, pid: {}",
                    style(pid).blue()
                );
            }
            ProcessCommand::Stop => {
                prettyln!("Stopping gistit network node process...");
            }
            ProcessCommand::Status => {
                if Bridge::check_alive(&runtime_dir) {
                    prettyln!("Running");
                } else {
                    prettyln!("Not running");
                }
            }
            // Not a process instruction
            ProcessCommand::Skip => {
                if let Self::InnerData {
                    maybe_file: Some(_file),
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

fn get_runtime_dir() -> Result<PathBuf> {
    let dirs = BaseDirs::new().ok_or(ErrorKind::Unknown)?;
    Ok(dirs
        .runtime_dir()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::temp_dir()))
}

fn spawn(runtime_dir: &Path, seed: &str, host: &str, port: &str) -> Result<u32> {
    let stdout = fs::File::create(runtime_dir.join("gistit.out"))?;
    let daemon = "/home/fabricio7p/Documents/Projects/gistit/target/debug/gistit-daemon";
    let child = Command::new(daemon)
        .args(["--seed", seed])
        .args(["--runtime-dir", runtime_dir.to_string_lossy().as_ref()])
        .args(["--host", host])
        .args(["--port", port])
        .stdout(stdout)
        .spawn()?;

    Ok(child.id())
}
