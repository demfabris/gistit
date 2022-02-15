use std::fs;
use std::process::{Command, Stdio};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;

use gistit_proto::{ipc, Instruction};
use gistit_reference::project;

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
        let runtime_dir = project::path::runtime()?;
        let mut bridge = gistit_ipc::client(&runtime_dir)?;

        match config.command {
            ProcessCommand::Start => {
                if bridge.alive() {
                    bridge.connect_blocking()?;
                    bridge.send(Instruction::request_status()).await?;

                    if let ipc::instruction::Kind::StatusResponse(response) =
                        bridge.recv().await?.expect_response()?
                    {
                        format_daemon_status(&response);
                    }
                    return Ok(());
                }

                progress!("Starting gistit node");
                let pid = {
                    let stdout = fs::File::create(runtime_dir.join("gistit.log"))?;
                    let daemon =
                        "/home/fabricio7p/Documents/Projects/gistit/target/debug/gistit-daemon";
                    Command::new(daemon)
                        .arg("--bootstrap")
                        .stderr(stdout)
                        .stdout(Stdio::null())
                        .spawn()?
                        .id()
                };
                updateln!("Gistit node started, pid: {}", style(pid).blue());

                bridge.connect_blocking()?;
                bridge.send(Instruction::request_status()).await?;

                if let ipc::instruction::Kind::StatusResponse(ipc::instruction::StatusResponse {
                    peer_id,
                    ..
                }) = bridge.recv().await?.expect_response()?
                {
                    finish!(format!("\n    peer id: '{}'\n\n", style(peer_id).bold()));
                }
            }

            ProcessCommand::Stop => {
                progress!("Stopping");
                if bridge.alive() {
                    fs::remove_file(runtime_dir.join("gistit.log"))?;

                    bridge.connect_blocking()?;
                    bridge.send(Instruction::request_shutdown()).await?;
                    updateln!("Stopped");
                    finish!("");
                } else {
                    interruptln!();
                    errorln!("gistit node is not running");
                }
            }

            ProcessCommand::Status => {
                progress!("Requesting status");
                if bridge.alive() {
                    bridge.connect_blocking()?;
                    bridge.send(Instruction::request_status()).await?;

                    if let ipc::instruction::Kind::StatusResponse(response) =
                        bridge.recv().await?.expect_response()?
                    {
                        format_daemon_status(&response);
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

fn format_daemon_status(response: &ipc::instruction::StatusResponse) {
    let ipc::instruction::StatusResponse {
        peer_id,
        listeners,
        peer_count,
        pending_connections,
        hosting,
    } = response;

    updateln!("Running status");
    finish!(format!(
        r#"
    peer id: '{}'
    hosting: {} gistit
    peers: {}
    pending connections: {}
    listening on: {:?}
        "#,
        style(peer_id).bold(),
        hosting,
        style(peer_count).blue(),
        pending_connections,
        listeners
    ));
}
