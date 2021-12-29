//! The host module
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use clap::ArgMatches;
use console::Style;
use daemonize::{Daemonize, DaemonizeError};
use directories::BaseDirs;

use lib_gistit::encrypt::{HashedSecret, Secret};
use lib_gistit::errors::internal::InternalError;
use lib_gistit::errors::io::IoError;
use lib_gistit::file::{File, FileReady};
use lib_gistit::network::NetworkDaemon;
use lib_gistit::{Error, Result};

use crate::dispatch::Dispatch;
use crate::gistit_line_out;
use crate::params::{HostParams, Params};

const UNIX_SIGKILL: i32 = 9;
const UNIX_SIGNOTHING: i32 = 0;

/// The Send action runtime parameters
#[derive(Debug, Clone)]
pub struct Action {
    /// Start p2p background process
    pub start: Option<&'static str>,
    /// Wether or not to save peers
    pub persist: bool,
    /// Address to listen for connections
    pub listen: &'static str,
    /// Stop p2p background process
    pub stop: bool,
    /// Display background process status
    pub status: bool,
    /// The secret key to protect your gistits.
    pub secret: Option<&'static str>,
    /// The file to be hosted.
    pub file: Option<&'static OsStr>,
    /// The private network to join
    pub join: Option<&'static str>,
}

impl Action {
    /// Parse [`ArgMatches`] into the dispatchable Host action.
    /// Here we also merge user settings while keeping this order of priority:
    /// arguments > local settings file > app defaults
    ///
    /// # Errors
    ///
    /// Fails with argument errors
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        // merge settings

        Ok(Box::new(Self {
            start: args.value_of("start"),
            persist: args.is_present("persist"),
            listen: args.value_of("listen").expect("to have default value"),
            stop: args.is_present("stop"),
            status: args.is_present("status"),
            secret: args.value_of("secret"),
            file: args.value_of_os("file"),
            join: args.value_of("join"),
        }))
    }
}

pub enum ProcessCommand {
    StartEncrypted(&'static str),
    Stop,
    Status,
    Skip,
}

pub struct Config {
    process_command: ProcessCommand,
    maybe_secret: Option<HashedSecret>,
    maybe_file: Option<Box<dyn FileReady + Send + Sync>>,
    maybe_join: Option<&'static str>,
}

impl Config {
    #[must_use]
    pub fn new(
        process_command: ProcessCommand,
        maybe_secret: Option<HashedSecret>,
        maybe_file: Option<Box<dyn FileReady + Send + Sync>>,
        maybe_join: Option<&'static str>,
    ) -> Self {
        Self {
            process_command,
            maybe_secret,
            maybe_file,
            maybe_join,
        }
    }
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&self) -> Result<Self::InnerData> {
        let params = Params::from_host(self).check_consume()?;
        let command = match (self.start, self.stop, self.status) {
            (Some(password), false, false) => ProcessCommand::StartEncrypted(password),
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
                gistit_line_out!("Encrypting...");

                let encrypted_file = file.into_encrypted(secret_str).await?;
                (Some(Box::new(encrypted_file)), Some(hashed_secret))
            } else {
                (Some(Box::new(file)), None)
            }
        } else {
            (None, None)
        };

        Ok(Config::new(
            command,
            maybe_hashed_secret,
            maybe_file,
            params.encoded_multiaddr,
        ))
    }

    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        let runtime_dir = get_runtime_dir()?;
        let cache_dir = runtime_dir.join("gistit_peers");

        if !Path::exists(&cache_dir) {
            std::fs::create_dir(&cache_dir)?;
        }

        match config.process_command {
            ProcessCommand::StartEncrypted(password) => {
                gistit_line_out!("Starting gistit network node process...");
                spawn_network_node_daemon(password, &cache_dir, self.persist, self.listen).await?;
            }
            ProcessCommand::Stop => {
                gistit_line_out!("Stopping gistit network node process...");
                signal_network_node_daemon(UNIX_SIGKILL)?;
            }
            ProcessCommand::Status => {
                if signal_network_node_daemon(UNIX_SIGNOTHING).is_ok() {
                    gistit_line_out!("Running");
                } else {
                    gistit_line_out!("Not running");
                }
            }
            // Not a process instruction, that means either add a peer or host a new gistit.
            ProcessCommand::Skip => match config {
                Self::InnerData {
                    maybe_join: Some(encoded_peer_id),
                    ..
                } => {
                    let peer_dir = Path::new(&cache_dir).join(encoded_peer_id);
                    if Path::exists(&peer_dir) {
                        gistit_line_out!("You're already connected to this peer");
                    } else {
                        gistit_line_out!("Added peer, check daemon outpt");
                        std::fs::create_dir(&peer_dir)?;
                    }
                }
                Self::InnerData {
                    maybe_file: Some(file),
                    ..
                } => {
                    todo!()
                }
                _ => (),
            },
        };
        Ok(())
    }
}

#[cfg(target_family = "unix")]
fn get_runtime_dir() -> Result<PathBuf> {
    let dirs = BaseDirs::new().ok_or_else(|| {
        Error::Internal(InternalError::Other("No valid home directory".to_owned()))
    })?;
    Ok(dirs
        .runtime_dir()
        .unwrap_or_else(|| Path::new("/tmp"))
        .to_path_buf())
}

#[cfg(target_family = "unix")]
async fn spawn_network_node_daemon(
    password: &'static str,
    cache_dir: &Path,
    persist_peers: bool,
    listen_addr: &'static str,
) -> Result<()> {
    let runtime_dir = get_runtime_dir()?;
    let daemon_out = std::fs::File::create(runtime_dir.join("gistit_node.out"))?;
    let daemon = Daemonize::new()
        .pid_file(runtime_dir.join("gistit_node.pid"))
        .stdout(daemon_out.try_clone()?)
        .stderr(daemon_out)
        .start();

    match daemon {
        Ok(network) => {
            NetworkDaemon::new(password, cache_dir)
                .await?
                .listen(listen_addr)
                .persist(persist_peers)
                .run()
                .await;
            Ok(())
        }
        Err(err) => match err {
            DaemonizeError::LockPidfile(pidf) => Err(Error::IO(IoError::ProcessSpawn(format!(
                "Process is already running... ({})",
                pidf
            )))),
            _ => Err(Error::IO(IoError::ProcessSpawn(
                "Failed to initialize daemon".to_owned(),
            ))),
        },
    }
}

#[cfg(target_family = "unix")]
fn signal_network_node_daemon(sig: i32) -> Result<()> {
    let runtime_dir = get_runtime_dir()?;
    let pid = std::fs::read_to_string(runtime_dir.join("gistit_node.pid"))?
        .parse::<i32>()
        .map_err(|_| {
            Error::Internal(InternalError::Other(
                "Process pid file is corrupted".to_owned(),
            ))
        })?;

    let res = unsafe { libc::kill(pid, sig) };

    if res == -1 {
        Err(Error::IO(IoError::ProcessStop(
            "Signal to gistit network node process failed. Is it running?".to_owned(),
        )))
    } else {
        Ok(())
    }
}

#[cfg(target_family = "windows")]
fn spawn_network_node_daemon(password: &str) -> Result<()> {
    Ok(())
}
