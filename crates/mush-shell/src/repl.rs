use clap::{Parser, Subcommand};
use std::path::PathBuf;
use mush_core::Response;
use mush_core::transport::TransportCommand;
use mush_core::zmq_client::ZmqClient;
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

#[derive(Subcommand, Debug)]
enum MidiCommand {
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
        Ok(cli) => match cli.domain {
            Domain::Version { long } => {
                if long {
                    Response::Output(version::long())
                } else {
                    Response::Output(version::short())
                }
            }

            Domain::Audio { command } => match command {
                AudioCommand::Hosts => send(client, TransportCommand::AudioHosts).await,

                AudioCommand::Host { name } => {
                    send(client, TransportCommand::AudioHost(name)).await
                }

                AudioCommand::Devices => {
                    send(client, TransportCommand::AudioDevices).await
                }

                AudioCommand::Device { name } => {
                    send(client, TransportCommand::AudioDevice(name)).await
                }

                AudioCommand::Load { path } => {
                    send(client, TransportCommand::LoadMidi(path)).await
                }

                AudioCommand::Save { path } => {
                    send(client, TransportCommand::Save(path)).await
                }

                AudioCommand::Play => send(client, TransportCommand::Play).await,

                AudioCommand::Stop => send(client, TransportCommand::Stop).await,

                AudioCommand::Record { path: _ } => {
                    Response::Output("audio record: not yet implemented".into())
                }

                AudioCommand::Monitor => {
                    Response::Output("audio monitor: not yet implemented".into())
                }
            },

            Domain::Midi { command } => match command {
                MidiCommand::Ports => send(client, TransportCommand::MidiDevices).await,

                MidiCommand::Port { name } => {
                    send(client, TransportCommand::MidiPort(name)).await
                }

                MidiCommand::Load { path } => {
                    let cmd = match path.extension().and_then(|e| e.to_str()) {
                        Some("mudl") => TransportCommand::LoadMudl(path),
                        _            => TransportCommand::LoadMidi(path),
                    };
                    send(client, cmd).await
                }

                MidiCommand::Save { path } => {
                    send(client, TransportCommand::Save(path)).await
                }

                MidiCommand::Play => send(client, TransportCommand::Play).await,

                MidiCommand::Stop => send(client, TransportCommand::Stop).await,

                MidiCommand::Record { path } => {
                    send(client, TransportCommand::Record { start: true, path }).await
                }

                MidiCommand::Monitor { path } => {
                    send(client, TransportCommand::Monitor { path }).await
                }

                MidiCommand::Send { args } => {
                    let joined = args.join(" ");
                    match mush_midi_codec::parse(&joined) {
                        Ok(bytes) => {
                            send(client, TransportCommand::SendMessage(bytes)).await
                        }
                        Err(e) => Response::Output(format!("error: {e}")),
                    }
                }
            },
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