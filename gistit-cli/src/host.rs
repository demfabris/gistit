//! The host module
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use async_trait::async_trait;
use clap::ArgMatches;
use directories::BaseDirs;

use lib_gistit::encrypt::{HashedSecret, Secret};
use lib_gistit::file::{File, FileReady};
use lib_gistit::ipc::Bridge;

use crate::dispatch::Dispatch;
use crate::params::Params;
use crate::{gistit_line_out, ErrorKind, Result};

const UNIX_SIGKILL: i32 = 9;
const UNIX_SIGNOTHING: i32 = 0;

#[derive(Debug, Clone)]
pub struct Action {
    pub start: bool,
    pub clipboard: bool,
    pub seed: Option<&'static str>,
    pub listen: &'static str,
    pub stop: bool,
    pub status: bool,
    pub secret: Option<&'static str>,
    pub file: Option<&'static OsStr>,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        // merge settings

        Ok(Box::new(Self {
            start: args.is_present("start"),
            clipboard: args.is_present("clipboard"),
            seed: args.value_of("seed"),
            // SAFETY: Has default value
            listen: unsafe { args.value_of("listen").unwrap_unchecked() },
            stop: args.is_present("stop"),
            status: args.is_present("status"),
            secret: args.value_of("secret"),
            file: args.value_of_os("file"),
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
        let mut node_hash: &str;
        let (host, port) = self.listen.split_once(':').ok_or(ErrorKind::Argument)?;

        if !Path::exists(&runtime_dir) {
            std::fs::create_dir(&runtime_dir)?;
        }

        let bridge = Bridge::bounded(&runtime_dir)?;

        match config.process_command {
            ProcessCommand::StartWithSeed(seed) => {
                gistit_line_out!("Starting gistit network node process with seed...");
                spawn_daemon_from_args(&runtime_dir, seed, host, port)?;
            }
            ProcessCommand::Start => {
                gistit_line_out!("Starting gistit network node process...");
                spawn_daemon_from_args(&runtime_dir, "none", host, port)?;
            }
            ProcessCommand::Stop => {
                gistit_line_out!("Stopping gistit network node process...");
                bridge.tx.send(b"asdiuauhsduhas").await?;
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
    let dirs = BaseDirs::new().ok_or(ErrorKind::Unknown)?;
    Ok(dirs
        .runtime_dir()
        .unwrap_or_else(|| Path::new("/tmp"))
        .to_path_buf())
}

fn spawn_daemon_from_args(runtime_dir: &Path, seed: &str, host: &str, port: &str) -> Result<()> {
    let stdout = std::fs::File::create(runtime_dir.join("gistit.out"))?;
    let daemon = "/home/fabricio7p/Documents/Projects/gistit/core/target/debug/daemon";
    let child = Command::new(daemon)
        .args(["--seed", seed])
        .args(["--runtime-dir", runtime_dir.to_string_lossy().as_ref()])
        .args(["--host", host])
        .args(["--port", port])
        .stdout(stdout)
        .spawn()?;

    Ok(())
}
