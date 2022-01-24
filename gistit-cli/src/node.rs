//! The host module
use std::ffi::OsStr;
use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use directories::BaseDirs;

use lib_gistit::file::File;
use lib_gistit::ipc::{self, Instruction, ServerResponse};

use crate::dispatch::Dispatch;
use crate::params::Check;
use crate::{prettyln, ErrorKind, Result};

#[derive(Debug, Clone)]
pub struct Action {
    pub file: Option<&'static OsStr>,
    pub start: Option<&'static str>,
    pub join: Option<&'static str>,
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
            join: args.value_of("join"),
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
    Join(&'static str),
    Stop,
    Status,
    Other,
}

pub struct Config {
    command: ProcessCommand,
    maybe_file: Option<File>,
    host: Ipv4Addr,
    port: u16,
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&'static self) -> Result<Self::InnerData> {
        <Self as Check>::check(self)?;

        let command = match (self.start, self.join, self.stop, self.status) {
            (Some(seed), None, false, false) => ProcessCommand::Start(seed),
            (None, Some(address), false, false) => ProcessCommand::Join(address),
            (None, None, true, false) => ProcessCommand::Stop,
            (None, None, false, true) => ProcessCommand::Status,
            (_, _, _, _) => ProcessCommand::Other,
        };

        let maybe_file = if let Some(file) = self.file {
            let path = Path::new(file);
            Some(File::from_path(path)?)
        } else {
            None
        };

        // SAFETY: Previously checked in [`Check::check`]
        let (host, port) = unsafe {
            (
                self.host.parse::<Ipv4Addr>().unwrap_unchecked(),
                self.port.parse::<u16>().unwrap_unchecked(),
            )
        };

        let config = Config {
            command,
            maybe_file,
            host,
            port,
        };

        Ok(config)
    }

    async fn dispatch(&'static self, config: Self::InnerData) -> Result<()> {
        let runtime_dir = get_runtime_dir()?;
        let mut bridge = ipc::client(&runtime_dir)?;

        match config.command {
            ProcessCommand::Start(seed) => {
                let pid = spawn(&runtime_dir, seed)?;
                prettyln!(
                    "Starting gistit network node process, pid: {}",
                    style(pid).blue()
                );

                bridge.connect_blocking()?;
                bridge
                    .send(Instruction::Listen {
                        host: config.host,
                        port: config.port,
                    })
                    .await?;

                if let Instruction::Response(ServerResponse::PeerId(id)) = bridge.recv().await? {
                    print_success(self.clipboard, id);
                }
            }
            ProcessCommand::Join(address) => {
                if !bridge.alive() {
                    prettyln!("Gistit node must be running to join a peer");
                } else {
                    bridge.connect_blocking()?;
                    bridge
                        .send(Instruction::Dial {
                            peer_id: address.to_owned(),
                        })
                        .await?;
                }
            }
            ProcessCommand::Stop => {
                prettyln!("Stopping gistit network node process...");
                fs::remove_file(runtime_dir.join("gistit.log"))?;
                bridge.connect_blocking()?;
                bridge.send(Instruction::Shutdown).await?;
            }
            ProcessCommand::Status => {
                if bridge.alive() {
                    bridge.connect_blocking()?;
                    bridge.send(Instruction::Status).await?;

                    if let Instruction::Response(ServerResponse::Status(status_str)) =
                        bridge.recv().await?
                    {
                        println!("{}", status_str);
                    }
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
                    bridge.connect_blocking()?;
                    bridge
                        .send(Instruction::Provide {
                            name: file.name(),
                            data: file.to_encoded_data(),
                        })
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

fn spawn(runtime_dir: &Path, seed: &str) -> Result<u32> {
    let stdout = fs::File::create(runtime_dir.join("gistit.log"))?;
    let daemon = "/home/fabricio7p/Documents/Projects/gistit/target/debug/gistit-daemon";
    let child = Command::new(daemon)
        .args(["--seed", seed])
        .args(["--runtime-dir", runtime_dir.to_string_lossy().as_ref()])
        .stderr(stdout)
        .stdout(Stdio::null())
        .spawn()?;

    Ok(child.id())
}

fn print_success(has_clipboard: bool, peer_id: String) {
    let clipboard_msg = if has_clipboard {
        "(copied to clipboard)".to_owned()
    } else {
        "".to_owned()
    };
    println!(
        r#"
SUCCESS:
    peer id: {} {}
"#,
        peer_id,
        style(clipboard_msg).italic()
    );
}
