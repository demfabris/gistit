use std::fs;
use std::net::Ipv4Addr;
use std::process::{Command, Stdio};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;

use gistit_ipc::{self, Instruction, ServerResponse};
use gistit_reference::dir;

use crate::arg::app;
use crate::dispatch::Dispatch;
use crate::param::check;
use crate::{errorln, finish, interruptln, progress, updateln, warnln, Error, Result};

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Action {
    pub start: bool,
    pub stop: bool,
    pub status: bool,
    pub host: &'static str,
    pub port: &'static str,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        Ok(Box::new(Self {
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

        let command = match (self.start, self.stop, self.status) {
            (true, false, false) => ProcessCommand::Start,
            (false, true, false) => ProcessCommand::Stop,
            (false, false, true) => ProcessCommand::Status,
            (_, _, _) => {
                app().print_help()?;
                std::process::exit(0);
            }
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
        let runtime_dir = dir::runtime()?;
        let mut bridge = gistit_ipc::client(&runtime_dir)?;

        match config.command {
            // Start network daemon / check running status
            ProcessCommand::Start => {
                if bridge.alive() {
                    finish!("Running"); // TODO: change this to status msg
                    return Ok(());
                }

                progress!("Starting gistit node");
                let pid = {
                    let stdout = fs::File::create(runtime_dir.join("gistit.log"))?;
                    let daemon =
                        "/home/fabricio7p/Documents/Projects/gistit/target/debug/gistit-daemon";
                    Command::new(daemon)
                        .stderr(stdout)
                        .stdout(Stdio::null())
                        .spawn()?
                        .id()
                };

                bridge.connect_blocking()?;
                bridge
                    .send(Instruction::Listen {
                        host: config.host,
                        port: config.port,
                    })
                    .await?;

                updateln!("Gistit node started, pid: {}", style(pid).blue());

                if let Instruction::Response(ServerResponse::PeerId(id)) = bridge.recv().await? {
                    finish!(format!("\n    peer id: '{}'\n\n", style(id).bold(),));
                }
            }

            // Stop network daemon process
            ProcessCommand::Stop => {
                progress!("Stopping");
                fs::remove_file(runtime_dir.join("gistit.log"))?;

                bridge.connect_blocking()?;
                bridge.send(Instruction::Shutdown).await?;
                updateln!("Stopped");
                finish!("");
            }

            // Check network status
            ProcessCommand::Status => {
                progress!("Requesting status");
                if bridge.alive() {
                    bridge.connect_blocking()?;
                    bridge.send(Instruction::Status).await?;

                    if let Instruction::Response(ServerResponse::Status(status_str)) =
                        bridge.recv().await?
                    {
                        updateln!("Requested status");
                        warnln!("{}", status_str);
                    }
                } else {
                    interruptln!();
                    errorln!("gistit node is not running");
                }
            }
        };
        Ok(())
    }
}
