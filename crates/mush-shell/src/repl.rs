use clap::{Parser, Subcommand};
use std::path::PathBuf;

use mush_core::transport::TransportCommand;
use mush_core::zmq_client::ZmqClient;
use mush_core::Response;

use crate::version;

#[derive(Parser, Debug)]
#[command(no_binary_name = true, name = "mush")]
struct Cli {
    #[command(subcommand)]
    domain: Domain,
}

#[derive(Subcommand, Debug)]
enum Domain {
    /// Show mush version
    Version {
        /// Show extended build information
        #[arg(long)]
        long: bool,
    },
    /// Audio commands
    Audio {
        #[command(subcommand)]
        command: AudioCommand,
    },
    /// MIDI commands
    Midi {
        #[command(subcommand)]
        command: MidiCommand,
    },
}

#[derive(Subcommand, Debug)]
enum AudioCommand {
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

#[derive(Subcommand, Debug)]
enum MidiCommand {
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
        Ok(cli) => match cli.domain {
            Domain::Version { long } => {
                if long {
                    Response::Output(version::long())
                } else {
                    Response::Output(version::short())
                }
            }
            Domain::Audio { command } => match command {
                AudioCommand::Devices => send(client, TransportCommand::AudioDevices).await,
                AudioCommand::Load { path } => send(client, TransportCommand::LoadMidi(path)).await,
                AudioCommand::Save { .. } => Response::Output("save: not yet implemented".into()),
                AudioCommand::Play => send(client, TransportCommand::Play).await,
                AudioCommand::Stop => send(client, TransportCommand::Stop).await,
                AudioCommand::Record => Response::Output("record: not yet implemented".into()),
                AudioCommand::Monitor => Response::Output("monitor: not yet implemented".into()),
            },
            Domain::Midi { command } => match command {
                MidiCommand::Devices => send(client, TransportCommand::MidiDevices).await,
                MidiCommand::Load { path } => send(client, TransportCommand::LoadMidi(path)).await,
                MidiCommand::Save { .. } => Response::Output("save: not yet implemented".into()),
                MidiCommand::Play => send(client, TransportCommand::Play).await,
                MidiCommand::Stop => send(client, TransportCommand::Stop).await,
                MidiCommand::Record => Response::Output("record: not yet implemented".into()),
                MidiCommand::Monitor => Response::Output("monitor: not yet implemented".into()),
            },
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