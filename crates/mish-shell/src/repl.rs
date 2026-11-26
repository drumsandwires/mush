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
    Play {
        /// Open the live view TUI during playback
        #[arg(short, long)]
        monitor: bool,
    },

    /// Start MIDI recording
    Record {
        /// Optional path to save the recording when stopped
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Open the live view TUI during recording
        #[arg(short, long)]
        monitor: bool,
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

        MishCommand::Play { monitor } => {
            let resp = send(client, TransportCommand::Play).await;
            if monitor {
                handle_live_view(
                    client,
                    "Playback",
                    mush_core::live_view::EventDirection::Out,
                    true,
                    TransportCommand::Stop,
                )
                .await
            } else {
                resp
            }
        }

        MishCommand::Stop => send(client, TransportCommand::Stop).await,

        MishCommand::Record { path, monitor } => {
            let resp = send(client, TransportCommand::Record { start: true, path }).await;
            if monitor {
                handle_live_view(
                    client,
                    "Recording",
                    mush_core::live_view::EventDirection::In,
                    true,
                    TransportCommand::Stop,
                )
                .await
            } else {
                resp
            }
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

/// Flush the ZMQ SUB backlog and wait for ActiveInputPort ack,
/// then enter the live view TUI.
///
/// The ZMQ SUB socket buffers all updates published since connect.
/// Before the Monitor command was sent, the conductor may have published
/// port assignments, device lists, and other updates. We drain these
/// silently until ActiveInputPort or Error arrives, or until timeout.
async fn handle_monitor(client: &mut ZmqClient, path: Option<PathBuf>) -> Response {
    if let Err(e) = client
        .send_command(&TransportCommand::Monitor { path })
        .await
    {
        return Response::Output(format!("error: {e}"));
    }

    // Drain stale ZMQ SUB backlog, wait for ActiveInputPort ack.
    // Timeout is 2 s — port open is synchronous, ack arrives in < 10 ms.
    let port_name = loop {
        match tokio::time::timeout(std::time::Duration::from_secs(2), client.recv_update()).await {
            Err(_) => {
                return Response::Output(
                    "error: monitor timed out waiting for port confirmation".into(),
                );
            }
            Ok(None) => {
                return Response::Output("error: conductor closed connection".into());
            }
            Ok(Some(EngineUpdate::Midi(MidiUpdate::Error(e)))) => {
                return Response::Output(format!("✗ {e}"));
            }
            Ok(Some(EngineUpdate::Midi(MidiUpdate::ActiveInputPort(Some(name))))) => {
                break name;
            }
            Ok(Some(_)) => continue, // stale — discard and keep waiting
        }
    };

    handle_live_view(
        client,
        &port_name,
        mush_core::live_view::EventDirection::In,
        false,
        TransportCommand::Stop,
    )
    .await
}

/// Enter the live view TUI with the given configuration.
async fn handle_live_view(
    client: &mut ZmqClient,
    label: &str,
    direction: mush_core::live_view::EventDirection,
    show_transport: bool,
    stop_command: TransportCommand,
) -> Response {
    let config = mush_core::live_view::LiveViewConfig {
        label: label.to_string(),
        direction,
        initial_view: mush_core::live_view::View::default(),
        stop_command,
        show_transport,
    };

    if let Err(e) = mush_core::live_view::run_live_view(client, config).await {
        return Response::Output(format!("error: live view failed: {e}"));
    }

    Response::Empty
}
