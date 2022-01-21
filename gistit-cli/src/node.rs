//! The host module
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use directories::BaseDirs;

use lib_gistit::file::File;
use lib_gistit::ipc::{Bridge, Instruction};

use crate::dispatch::Dispatch;
use crate::params::Check;
use crate::{prettyln, ErrorKind, Result};

#[derive(Debug, Clone)]
pub struct Action {
    pub file: Option<&'static OsStr>,
    pub start: Option<&'static str>,
    pub stop: bool,
    pub status: bool,
    pub host: &'static str,
    pub port: &'static str,
    pub clipboard: bool,
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
            file: args.value_of_os("FILE"),
        }))
    }
}

pub enum ProcessCommand {
    Start(&'static str),
    Stop,
    Status,
    Other,
}

pub struct Config {
    command: ProcessCommand,
    maybe_file: Option<File>,
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&'static self) -> Result<Self::InnerData> {
        <Self as Check>::check(self)?;

        let command = match (self.start, self.stop, self.status) {
            (Some(seed), false, false) => ProcessCommand::Start(seed),
            (None, true, false) => ProcessCommand::Stop,
            (None, false, true) => ProcessCommand::Status,
            (_, _, _) => ProcessCommand::Other,
        };

        // Construct `FileReady` if a file was provided, that means user wants to host a new file
        // and the background process should be running.
        let maybe_file = if let Some(file) = self.file {
            let path = Path::new(file);
            Some(File::from_path(path)?)
        } else {
            None
        };
        let config = Config {
            command,
            maybe_file,
        };

        Ok(config)
    }

    async fn dispatch(&'static self, config: Self::InnerData) -> Result<()> {
        let runtime_dir = get_runtime_dir()?;

        match config.command {
            ProcessCommand::Start(seed) => {
                let pid = spawn(&runtime_dir, seed, self.host, self.port)?;
                prettyln!(
                    "Starting gistit network node process, pid: {}",
                    style(pid).blue()
                );
            }
            ProcessCommand::Stop => {
                Bridge::connect(&runtime_dir)?
                    .send(Instruction::Shutdown)
                    .await?;
                prettyln!("Stopping gistit network node process...");
            }
            ProcessCommand::Status => {
                if Bridge::alive(&runtime_dir) {
                    prettyln!("Running");
                    // TODO: include more info from daemon
                } else {
                    prettyln!("Not running");
                }
            }
            ProcessCommand::Other => {
                if let Self::InnerData {
                    maybe_file: Some(file),
                    ..
                } = config
                {
                    prettyln!("Hosting file...");
                    Bridge::connect(&runtime_dir)?
                        .send(Instruction::File(file.to_encoded_data()))
                        .await?;
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
