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
use crate::{cleanln, errorln, finish, interruptln, progress, updateln, warnln, Error, Result};

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Action {
    pub start: bool,
    pub stop: bool,
    pub status: bool,
    pub attach: bool,
    // Hidden args
    host: &'static str,
    port: &'static str,
    maybe_dial: Option<&'static str>,
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
            host: args
                .value_of("host")
                .ok_or(Error::Argument("missing argument", "--host"))?,
            port: args
                .value_of("port")
                .ok_or(Error::Argument("missing argument", "--host"))?,
            maybe_dial: args.value_of("dial"),
        }))
    }
}

enum ProcessCommand {
    Start,
    Status,
    Stop,
    Attach,
}

pub struct Config {
    command: ProcessCommand,
    host: &'static str,
    port: &'static str,
    maybe_dial: Option<&'static str>,
    runtime_path: PathBuf,
    config_path: PathBuf,
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&self) -> Result<Self::InnerData> {
        progress!("Preparing");
        let command = match (self.start, self.stop, self.status, self.attach) {
            (true, false, false, _) => ProcessCommand::Start,
            (false, false, true, _) => ProcessCommand::Status,
            (false, true, false, false) => ProcessCommand::Stop,
            (false, false, false, true) => ProcessCommand::Attach,
            (_, _, _, _) => {
                app().print_help()?;
                std::process::exit(1);
            }
        };

        let (host, port) = check::host_port(self.host, self.port)?;
        let config = Config {
            command,
            host,
            port,
            maybe_dial: self.maybe_dial,
            runtime_path: path::runtime()?,
            config_path: path::config()?,
        };
        updateln!("Prepared");

        Ok(config)
    }

    #[allow(clippy::too_many_lines)]
    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        let mut bridge = gistit_ipc::client(&config.runtime_path)?;

        match config.command {
            ProcessCommand::Start => {
                if bridge.alive() {
                    bridge.connect_blocking()?;
                    bridge.send(Instruction::request_status()).await?;

                    if self.attach {
                        attach_to_log(&config.runtime_path)?;
                    }

                    if let ipc::instruction::Kind::StatusResponse(response) =
                        bridge.recv().await?.expect_response()?
                    {
                        format_daemon_status(&response);
                    }

                    return Ok(());
                }

                progress!("Starting gistit node");
                let pid = {
                    let stdout = fs::File::create(config.runtime_path.join("gistit.log"))?;
                    // FIXME: Fix this before release
                    let daemon =
                        "/home/fabricio7p/Documents/Projects/gistit/target/debug/gistit-daemon";

                    let cmd = Box::leak(Box::new(Command::new(daemon)))
                        .args(&["--host", config.host])
                        .args(&["--port", config.port])
                        .args(&["--runtime-path", &*config.runtime_path.to_string_lossy()])
                        .args(&["--config-path", &*config.config_path.to_string_lossy()])
                        .stderr(stdout)
                        .stdout(Stdio::null());

                    if let Some(addr) = config.maybe_dial {
                        warnln!("dialing address on init: {:?}", addr);
                        cmd.args(&["--dial", addr]);
                    }

                    cmd.spawn()?.id()
                };
                updateln!("Gistit node started, pid: {}", style(pid).blue());

                bridge.connect_blocking()?;
                bridge.send(Instruction::request_status()).await?;

                if let ipc::instruction::Kind::StatusResponse(ipc::instruction::StatusResponse {
                    peer_id,
                    ..
                }) = bridge.recv().await?.expect_response()?
                {
                    if self.attach {
                        attach_to_log(&config.runtime_path)?;
                    } else {
                        finish!(format!("\n    peer id: '{}'\n\n", style(peer_id).bold()));
                    }
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

            ProcessCommand::Attach => {
                attach_to_log(&config.runtime_path)?;
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

fn attach_to_log(runtime_path: &Path) -> Result<()> {
    let log_path = runtime_path.join("gistit.log");

    if let Ok(log) = fs::File::open(&log_path) {
        let mut reader = BufReader::new(&log);
        let mut buf = String::new();
        progress!(
            "Executing {}",
            style("(CTRL-C exits the process)").italic().dim()
        );

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
