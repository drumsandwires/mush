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
    /// List available MIDI output ports
    Ports,
    /// Get or set the active MIDI output port
    Port {
        /// Port name to select; omit to query the current port
        name: Option<String>,
    },
    /// Load a MIDI file (.mid or .mudl)
    Load { path: PathBuf },
    /// Save the current timeline
    Save { path: PathBuf },
    /// Start MIDI playback
    Play,
    /// Start MIDI recording
    Record {
        /// Optional path to save the recording when stopped
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Monitor live MIDI input
    Monitor {
        /// Optional autosave path for the pre-roll buffer
        path: Option<PathBuf>,
    },
    /// Stop playback or recording
    Stop,
    /// Send a raw MIDI message
    Send {
        /// Message in symbolic or numeric form, e.g. "NoteOn C4 mf" or "0x90 0x3C 0x64"
        #[arg(required = true, num_args = 1..)]
        args: Vec<String>,
    },
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

            MishCommand::Ports => send(client, TransportCommand::MidiDevices).await,

            MishCommand::Port { name } => {
                send(client, TransportCommand::MidiPort(name)).await
            }

            MishCommand::Load { path } => {
                let cmd = match path.extension().and_then(|e| e.to_str()) {
                    Some("mudl") => TransportCommand::LoadMudl(path),
                    _            => TransportCommand::LoadMidi(path),
                };
                send(client, cmd).await
            }

            MishCommand::Save { path } => send(client, TransportCommand::Save(path)).await,

            MishCommand::Play => send(client, TransportCommand::Play).await,

            MishCommand::Stop => send(client, TransportCommand::Stop).await,

            MishCommand::Record { path } => {
                send(client, TransportCommand::Record { start: true, path }).await
            }

            MishCommand::Monitor { path } => {
                send(client, TransportCommand::Monitor { path }).await
            }

            MishCommand::Send { args } => {
                let joined = args.join(" ");
                match mush_midi_codec::parse(&joined) {
                    Ok(bytes) => send(client, TransportCommand::SendMessage(bytes)).await,
                    Err(e)    => Response::Output(format!("error: {e}")),
                }
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