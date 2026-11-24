use clap::{Parser, Subcommand};
use mush_core::Response;
use mush_core::transport::TransportCommand;
use mush_core::update::{EngineUpdate, MidiUpdate};
use mush_core::zmq_client::ZmqClient;
use mush_tracing::tracing;

use std::path::PathBuf;

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

    /// List available MIDI ports (input, output, and thru)
    Ports,

    /// Show current MIDI port assignments (in and out)
    Port {
        #[command(subcommand)]
        direction: Option<PortDirection>,
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

#[derive(Subcommand, Debug)]
enum PortDirection {
    /// Set the active MIDI input port by name, index, or substring
    In {
        /// Port selector — name, integer index, or substring; omit to clear
        selector: Option<String>,
    },
    /// Set the active MIDI output port by name, index, or substring
    Out {
        /// Port selector — name, integer index, or substring; omit to clear
        selector: Option<String>,
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
        Err(e) => Response::Output(e.render().to_string()),
        Ok(cli) => dispatch(client, cli.command).await,
    }
}

#[tracing::instrument(skip(client), fields(cmd = ?command))]
async fn dispatch(client: &mut ZmqClient, command: MishCommand) -> Response {
    match command {
        MishCommand::Version { long } => {
            if long {
                Response::Output(version::long())
            } else {
                Response::Output(version::short())
            }
        }

        MishCommand::Ports => send(client, TransportCommand::MidiDevices).await,

        MishCommand::Port { direction } => match direction {
            None => send(client, TransportCommand::MidiPortGet).await,
            Some(PortDirection::In { selector }) => {
                send(client, TransportCommand::MidiInputPort(selector)).await
            }
            Some(PortDirection::Out { selector }) => {
                send(client, TransportCommand::MidiOutputPort(selector)).await
            }
        },

        MishCommand::Load { path } => {
            let cmd = match path.extension().and_then(|e| e.to_str()) {
                Some("mudl") => TransportCommand::LoadMudl(path),
                _ => TransportCommand::LoadMidi(path),
            };
            send(client, cmd).await
        }

        MishCommand::Save { path } => send(client, TransportCommand::Save(path)).await,

        MishCommand::Play => send(client, TransportCommand::Play).await,

        MishCommand::Stop => send(client, TransportCommand::Stop).await,

        MishCommand::Record { path } => {
            send(client, TransportCommand::Record { start: true, path }).await
        }

        MishCommand::Monitor { path } => handle_monitor(client, path).await,

        MishCommand::Send { args } => {
            let joined = args.join(" ");
            match mush_midi_codec::parse(&joined) {
                Ok(bytes) => send(client, TransportCommand::SendMessage(bytes)).await,
                Err(e) => Response::Output(format!("error: {e}")),
            }
        }
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

async fn handle_monitor(client: &mut ZmqClient, path: Option<PathBuf>) -> Response {
    // Send the Monitor command
    if let Err(e) = client
        .send_command(&TransportCommand::Monitor { path })
        .await
    {
        return Response::Output(format!("error: {e}"));
    }

    // Wait for ActiveInputPort ack before entering the event loop
    // so the user sees which port was opened (or an error).
    // Timeout after 2 s to avoid hanging on a silent conductor.
    let ack = tokio::time::timeout(std::time::Duration::from_secs(2), client.recv_update()).await;

    match ack {
        Err(_) => return Response::Output("error: monitor timed out waiting for port ack".into()),
        Ok(None) => return Response::Output("error: conductor closed connection".into()),
        Ok(Some(update)) => {
            if let Some(line) = mush_core::format::format_update(&update) {
                println!("{line}");
            }
            // If the conductor returned an error, stop here
            if matches!(update, EngineUpdate::Midi(MidiUpdate::Error(_))) {
                return Response::Empty;
            }
        }
    }

    println!("monitoring MIDI input — press Ctrl+C to stop");

    // Event loop — runs until Space/Enter
    loop {
        tokio::select! {
            _ = wait_for_stop() => {
                println!("stopped");
                let _ = client.send_command(&TransportCommand::Stop).await;
                break;
            }
            update = client.recv_update() => {
                match update {
                    None => {
                        println!("conductor disconnected");
                        break;
                    }
                    Some(update) => {
                        if let Some(line) = mush_core::format::format_update(&update) {
                            println!("{line}");
                        }
                    }
                }
            }
        }
    }

    Response::Empty
}

async fn wait_for_stop() {
    use tokio::io::AsyncReadExt;
    let mut stdin = tokio::io::stdin();
    let mut buf = [0u8; 1];
    loop {
        if stdin.read(&mut buf).await.is_err() {
            break;
        }
        if buf[0] == b' ' || buf[0] == b'\n' || buf[0] == b'\r' {
            break;
        }
    }
}
