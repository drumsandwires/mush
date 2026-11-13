use clap::{Parser, Subcommand};
use std::path::PathBuf;
use mush_core::Response;
use mush_core::transport::TransportCommand;
use mush_core::zmq_client::ZmqClient;
use crate::version;

#[derive(Parser, Debug)]
#[command(no_binary_name = true, name = "mash")]
struct Cli {
    #[command(subcommand)]
    command: MashCommand,
}

#[derive(Subcommand, Debug)]
enum MashCommand {
    /// Show mash version
    Version {
        /// Show extended build information
        #[arg(long)]
        long: bool,
    },
    /// List available audio hosts
    Hosts,
    /// Get or set the active audio host
    Host {
        /// Host name to select; omit to query the current host
        name: Option<String>,
    },
    /// List available audio devices on the active host
    Devices,
    /// Get or set the active audio output device
    Device {
        /// Device name to select; omit to query the current device
        name: Option<String>,
    },
    /// Load an audio file
    Load { path: PathBuf },
    /// Save the current session
    Save { path: PathBuf },
    /// Start audio playback
    Play,
    /// Start recording
    Record {
        /// Optional path to save the recording when stopped
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Monitor live audio
    Monitor,
    /// Stop playback or recording
    Stop,
}

pub async fn evaluate(client: &mut ZmqClient, input: &str) -> Response {
    if input.is_empty() {
        return Response::Empty;
    }

    let tokens = match shlex::split(input) {
        Some(t) => t,
        None => {
            return Response::Output(
                "error: malformed input (unclosed quote or trailing backslash)".into(),
            );
        }
    };

    match Cli::try_parse_from(tokens) {
        Ok(cli) => match cli.command {
            MashCommand::Version { long } => {
                if long {
                    Response::Output(version::long())
                } else {
                    Response::Output(version::short())
                }
            }

            MashCommand::Hosts => send(client, TransportCommand::AudioHosts).await,

            MashCommand::Host { name } => {
                send(client, TransportCommand::AudioHost(name)).await
            }

            MashCommand::Devices => send(client, TransportCommand::AudioDevices).await,

            MashCommand::Device { name } => {
                send(client, TransportCommand::AudioDevice(name)).await
            }

            MashCommand::Load { path } => {
                send(client, TransportCommand::LoadMidi(path)).await
            }

            MashCommand::Save { path } => send(client, TransportCommand::Save(path)).await,

            MashCommand::Play => send(client, TransportCommand::Play).await,

            MashCommand::Stop => send(client, TransportCommand::Stop).await,

            MashCommand::Record { path: _ } => {
                Response::Output("record: not yet implemented".into())
            }

            MashCommand::Monitor => {
                Response::Output("monitor: not yet implemented".into())
            }
        },
        Err(e) => Response::Output(e.render().to_string()),
    }
}

async fn send(client: &mut ZmqClient, cmd: TransportCommand) -> Response {
    tracing::debug!(?cmd, "send_command called");
    match client.send_command(&cmd).await {
        Ok(()) => {
            tracing::debug!(?cmd, "send_command ok");
            Response::Empty
        }
        Err(ref e) => {
            tracing::error!(?cmd, error = e, "send_command failed");
            Response::Output(format!("error: {e}"))
        }
    }
}