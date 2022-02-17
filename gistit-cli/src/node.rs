use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;

use gistit_project::path;
use gistit_proto::{ipc, Instruction};

use crate::arg::app;
use crate::dispatch::Dispatch;
use crate::param::check;
use crate::{cleanln, errorln, finish, interruptln, progress, updateln, Error, Result};

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Action {
    pub start: bool,
    pub stop: bool,
    pub status: bool,
    pub attach: bool,
    // Hidden args
    dial: Option<&'static str>,
    host: &'static str,
    port: &'static str,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        Ok(Box::new(Self {
            start: args.is_present("start"),
            stop: args.is_present("stop"),
            status: args.is_present("status"),
            attach: args.is_present("attach"),
            dial: args.value_of("dial"),
            host: args
                .value_of("host")
                .ok_or(Error::Argument("missing argument", "--host"))?,
            port: args
                .value_of("port")
                .ok_or(Error::Argument("missing argument", "--host"))?,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProcessCommand {
    Start,
    Status,
    Stop,
    Attach,
    Dial(&'static str),
}

pub struct Config {
    commands: Vec<ProcessCommand>,
    host: &'static str,
    port: &'static str,
    runtime_path: PathBuf,
    config_path: PathBuf,
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&self) -> Result<Self::InnerData> {
        progress!("Preparing");
        let mut commands: Vec<ProcessCommand> = Vec::new();
        match (self.start, self.stop, self.status, self.attach, self.dial) {
            // Matching:
            // - start
            // - start [attach]
            // - start [dial]
            // - start [attach] [dial]
            (true, false, false, attach, dial) => {
                commands.push(ProcessCommand::Start);

                if let Some(addr) = dial {
                    commands.push(ProcessCommand::Dial(addr));
                }

                if attach {
                    commands.push(ProcessCommand::Attach);
                }
            }
            // Matching:
            // - status
            // - status [attach]
            // - status [dial]
            // - status [attach] [dial]
            (false, false, true, attach, dial) => {
                commands.push(ProcessCommand::Status);

                if let Some(addr) = dial {
                    commands.push(ProcessCommand::Dial(addr));
                }

                if attach {
                    commands.push(ProcessCommand::Attach);
                }
            }
            // Matching:
            // - attach
            // - attach [dial]
            (false, false, false, true, dial) => {
                commands.push(ProcessCommand::Attach);

                if let Some(addr) = dial {
                    commands.push(ProcessCommand::Dial(addr));
                }
            }
            // Matching:
            // - dial
            // - dial [attach]
            (false, false, false, attach, Some(addr)) => {
                commands.push(ProcessCommand::Dial(addr));

                if attach {
                    commands.push(ProcessCommand::Attach);
                }
            }
            // Matching:
            // - stop
            (false, true, false, false, None) => commands.push(ProcessCommand::Stop),
            // No match. Clap should not let this branch happen
            (_, _, _, _, _) => {
                app().print_help()?;
                std::process::exit(1);
            }
        };

        let (host, port) = check::host_port(self.host, self.port)?;
        let config = Config {
            commands,
            host,
            port,
            runtime_path: path::runtime()?,
            config_path: path::config()?,
        };
        updateln!("Prepared");

        Ok(config)
    }

    #[allow(clippy::too_many_lines)]
    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        let mut bridge = gistit_ipc::client(&config.runtime_path)?;

        for command in &config.commands {
            match command {
                ProcessCommand::Start => {
                    if bridge.alive() {
                        bridge.connect_blocking()?;
                        bridge.send(Instruction::request_status()).await?;

                        if let ipc::instruction::Kind::StatusResponse(response) =
                            bridge.recv().await?.expect_response()?
                        {
                            format_daemon_status(&response);
                        }

                        continue;
                    }

                    progress!("Starting gistit node");
                    let pid = {
                        let stdout = fs::File::create(config.runtime_path.join("gistit.log"))?;
                        // FIXME: Fix this before release
                        let daemon = "gistit-daemon";

                        Command::new(daemon)
                            .args(&["--host", config.host])
                            .args(&["--port", config.port])
                            .args(&["--runtime-path", &*config.runtime_path.to_string_lossy()])
                            .args(&["--config-path", &*config.config_path.to_string_lossy()])
                            .arg("--bootstrap")
                            .stderr(stdout)
                            .stdout(Stdio::null())
                            .spawn()?
                            .id()
                    };

                    updateln!("Gistit node started, pid: {}", style(pid).blue());
                    bridge.connect_blocking()?;
                    bridge.send(Instruction::request_status()).await?;

                    if let ipc::instruction::Kind::StatusResponse(
                        ipc::instruction::StatusResponse { peer_id, .. },
                    ) = bridge.recv().await?.expect_response()?
                    {
                        cleanln!(format!("\n    peer id: '{}'\n\n", style(peer_id).bold()));
                    }
                }

                ProcessCommand::Stop => {
                    progress!("Stopping");
                    if bridge.alive() {
                        fs::remove_file(config.runtime_path.join("gistit.log"))?;

                        bridge.connect_blocking()?;
                        bridge.send(Instruction::request_shutdown()).await?;
                        updateln!("Stopped");
                        finish!("");
                    } else {
                        interruptln!();
                        errorln!("gistit node is not running");
                        std::process::exit(1);
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
                        std::process::exit(1);
                    }
                }

                ProcessCommand::Dial(addr) => {
                    progress!("Dialing");
                    if bridge.alive() {
                        bridge.connect_blocking()?;
                        bridge
                            .send(Instruction::request_dial((*addr).to_string()))
                            .await?;
                        updateln!("Dialed");
                    } else {
                        interruptln!();
                        errorln!("gistit node is not running");
                        std::process::exit(1);
                    }
                }

                ProcessCommand::Attach => {
                    attach_to_log(
                        &config.runtime_path,
                        config
                            .commands
                            .iter()
                            .any(|cmd| *cmd == ProcessCommand::Start),
                    )?;
                }
            };
        }
        finish!("");
        Ok(())
    }
}

fn format_daemon_status(response: &ipc::instruction::StatusResponse) {
    let ipc::instruction::StatusResponse {
        peer_id,
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
        "#,
        style(peer_id).bold(),
        hosting,
        style(peer_count).blue(),
        pending_connections,
    ));
}

fn attach_to_log(runtime_path: &Path, linked: bool) -> Result<()> {
    let log_path = runtime_path.join("gistit.log");

    if let Ok(log) = fs::File::open(&log_path) {
        let mut reader = BufReader::new(&log);
        let mut buf = String::new();

        if linked {
            progress!(
                "Executing {}",
                style("(CTRL-C exits the process)").italic().dim()
            );
        } else {
            finish!("");
        }

        loop {
            let bytes = reader.read_line(&mut buf)?;
            if bytes > 0 {
                cleanln!(buf);
                buf = String::new();
            } else {
                sleep(Duration::from_millis(500));
            }
        }
    } else {
        interruptln!();
        errorln!("can't attach to log file, is it running?");
    }

    Ok(())
}
