use clap::{Parser, Subcommand};
use std::path::PathBuf;

use mush_core::transport::TransportCommand;
use mush_core::zmq_client::ZmqClient;
use mush_core::Response;

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
    /// List available audio devices
    Devices,
    /// Load an audio file
    Load { path: PathBuf },
    /// Save current session
    Save { path: PathBuf },
    /// Start audio playback
    Play,
    /// Start recording
    Record,
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
            MashCommand::Devices => send(client, TransportCommand::AudioDevices).await,
            MashCommand::Load { path } => send(client, TransportCommand::LoadMidi(path)).await,
            MashCommand::Save { .. } => Response::Output("save: not yet implemented".into()),
            MashCommand::Play => send(client, TransportCommand::Play).await,
            MashCommand::Stop => send(client, TransportCommand::Stop).await,
            MashCommand::Record => Response::Output("record: not yet implemented".into()),
            MashCommand::Monitor => Response::Output("monitor: not yet implemented".into()),
        },
        Err(e) => Response::Output(e.render().to_string()),
    }
}

async fn send(client: &mut ZmqClient, cmd: TransportCommand) -> Response {
    if let Err(e) = client.send_command(&cmd).await {
        return Response::Output(format!("error: {e}"));
    }
    Response::Empty
}