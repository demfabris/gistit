use std::fs;
use std::process::{Command, Stdio};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;

use gistit_ipc::{self, Instruction, ServerResponse};
use gistit_reference::dir;

use crate::arg::app;
use crate::dispatch::Dispatch;
use crate::{errorln, finish, interruptln, progress, updateln, Result};

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Action {
    pub start: bool,
    pub stop: bool,
    pub status: bool,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static> {
        Box::new(Self {
            start: args.is_present("start"),
            stop: args.is_present("stop"),
            status: args.is_present("status"),
        })
    }
}

enum ProcessCommand {
    Start,
    Stop,
    Status,
}

pub struct Config {
    command: ProcessCommand,
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&self) -> Result<Self::InnerData> {
        progress!("Preparing");

        let command = match (self.start, self.stop, self.status) {
            (true, false, false) => ProcessCommand::Start,
            (false, true, false) => ProcessCommand::Stop,
            (false, false, true) => ProcessCommand::Status,
            (_, _, _) => {
                app().print_help()?;
                std::process::exit(0);
            }
        };
        let config = Config { command };
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
                    bridge.connect_blocking()?;
                    bridge.send(Instruction::Status).await?;
                    if let Instruction::Response(ServerResponse::Status {
                        listeners,
                        peer_count,
                        pending_connections,
                    }) = bridge.recv().await?
                    {
                        format_daemon_status(&listeners, peer_count, pending_connections);
                    }
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

                    if let Instruction::Response(ServerResponse::Status {
                        listeners,
                        peer_count,
                        pending_connections,
                    }) = bridge.recv().await?
                    {
                        format_daemon_status(&listeners, peer_count, pending_connections);
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

fn format_daemon_status(listeners: &Vec<String>, peer_count: usize, pending_connections: u32) {
    updateln!("Running status");
    finish!(format!(
        r#"
    peers: {}
    pending connections: {}
    listening on: {:?}
                            "#,
        style(peer_count).blue(),
        pending_connections,
        listeners
    ));
}
