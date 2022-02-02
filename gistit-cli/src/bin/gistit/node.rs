use std::fs;
use std::net::Ipv4Addr;
use std::path::Path;
use std::process::{Command, Stdio};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;

use gistit_ipc::{self, Instruction, ServerResponse};
use libgistit::project::{config_dir, runtime_dir};

use crate::dispatch::Dispatch;
use crate::param::check;
use crate::{progress, updateln, Error, Result};

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Action {
    pub start: bool,
    pub stop: bool,
    pub status: bool,
    pub host: &'static str,
    pub port: &'static str,
    pub join: Option<&'static str>,
    pub clipboard: bool,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        Ok(Box::new(Self {
            join: args.value_of("join"),
            clipboard: args.is_present("clipboard"),
            host: args
                .value_of("host")
                .ok_or(Error::Argument("missing argument", "--host"))?,
            port: args
                .value_of("port")
                .ok_or(Error::Argument("missing argument", "--port"))?,
            start: args.is_present("start"),
            stop: args.is_present("stop"),
            status: args.is_present("status"),
        }))
    }
}

enum ProcessCommand {
    Join(&'static str),
    Start,
    Stop,
    Status,
}

pub struct Config {
    command: ProcessCommand,
    host: Ipv4Addr,
    port: u16,
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&self) -> Result<Self::InnerData> {
        progress!("Preparing");
        let host = check::host(self.host)?;

        let port = check::port(self.port)?;

        let command = match (self.join, self.start, self.stop, self.status) {
            (Some(address), false, false, false) => ProcessCommand::Join(address),
            (None, true, false, false) => ProcessCommand::Start,
            (None, false, true, false) => ProcessCommand::Stop,
            (None, false, false, true) => ProcessCommand::Status,
            (_, _, _, _) => unreachable!(), // TODO: print app help
        };

        let config = Config {
            command,
            host,
            port,
        };
        updateln!("Prepared");

        Ok(config)
    }

    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        let runtime_dir = runtime_dir()?;
        let config_dir = config_dir()?;
        let mut bridge = gistit_ipc::client(&runtime_dir)?;

        match config.command {
            ProcessCommand::Start => {
                if bridge.alive() {
                    progress!("Running..."); // TODO: change this to status msg
                    return Ok(());
                }

                let pid = spawn(&runtime_dir, &config_dir)?;

                progress!(
                    "Starting gistit network node process, pid: {}",
                    style(pid).blue()
                );

                bridge.connect_blocking()?;
                bridge.send(Instruction::Listen {
                    host: config.host,
                    port: config.port,
                })?;

                if let Instruction::Response(ServerResponse::PeerId(id)) = bridge.recv()? {
                    print_success(self.clipboard, &id);
                }
            }
            ProcessCommand::Join(address) => {
                if bridge.alive() {
                    bridge.connect_blocking()?;
                    bridge.send(Instruction::Dial {
                        peer_id: address.to_owned(),
                    })?;
                } else {
                    progress!("Gistit node must be running to join a peer");
                }
            }
            ProcessCommand::Stop => {
                progress!("Stopping gistit network node process...");
                fs::remove_file(runtime_dir.join("gistit.log"))?;

                bridge.connect_blocking()?;
                bridge.send(Instruction::Shutdown)?;
            }
            ProcessCommand::Status => {
                if bridge.alive() {
                    bridge.connect_blocking()?;
                    bridge.send(Instruction::Status)?;

                    if let Instruction::Response(ServerResponse::Status(status_str)) =
                        bridge.recv()?
                    {
                        println!("{}", status_str);
                    }
                } else {
                    progress!("Not running");
                }
            }
        };
        Ok(())
    }
}

fn spawn(runtime_dir: &Path, config_dir: &Path) -> Result<u32> {
    let stdout = fs::File::create(runtime_dir.join("gistit.log"))?;
    let daemon = "/home/fabricio7p/Documents/Projects/gistit/target/debug/gistit-daemon";
    let child = Command::new(daemon)
        .args(["--runtime-dir", runtime_dir.to_string_lossy().as_ref()])
        .args(["--config-dir", config_dir.to_string_lossy().as_ref()])
        .stderr(stdout)
        .stdout(Stdio::null())
        .spawn()?;

    Ok(child.id())
}

fn print_success(has_clipboard: bool, peer_id: &str) {
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
