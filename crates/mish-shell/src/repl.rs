use clap::{Parser, Subcommand};
use std::path::PathBuf;

use mush_core::Response;
use mush_core::transport::TransportCommand;
use mush_core::zmq_client::ZmqClient;

use crate::version;

#[derive(Parser, Debug)]
#[command(no_binary_name = true, name = "mish")]
struct Cli {
    #[command(subcommand)]
    command: MishCommand,
}

#[derive(Subcommand, Debug)]
enum MishCommand {
    /// Show mish version
    Version {
        /// Show extended build information
        #[arg(long)]
        long: bool,
    },
    /// List available MIDI devices
    Devices,
    /// Load a MIDI file
    Load { path: PathBuf },
    /// Save current session
    Save { path: PathBuf },
    /// Start MIDI playback
    Play,
    /// Start MIDI recording
    Record,
    /// Monitor live MIDI
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
            MishCommand::Version { long } => {
                if long {
                    Response::Output(version::long())
                } else {
                    Response::Output(version::short())
                }
            }
            MishCommand::Devices => send(client, TransportCommand::MidiDevices).await,
            MishCommand::Load { path } => send(client, TransportCommand::LoadMidi(path)).await,
            MishCommand::Save { .. } => Response::Output("save: not yet implemented".into()),
            MishCommand::Play => send(client, TransportCommand::Play).await,
            MishCommand::Stop => send(client, TransportCommand::Stop).await,
            MishCommand::Record => Response::Output("record: not yet implemented".into()),
            MishCommand::Monitor => Response::Output("monitor: not yet implemented".into()),
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
