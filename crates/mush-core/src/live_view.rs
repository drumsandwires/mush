//! Live view TUI for MIDI event display during monitor, playback, and recording.
//!
//! # Entry point
//! [`run_live_view`] — takes over the terminal for the duration of a live
//! command and restores it exactly on exit.
//!
//! # Views
//! Switchable live with Tab:
//! - [`View::Raw`]      — raw hex + decimal bytes, e.g. `90 3C 5B`
//! - [`View::Symbolic`] — codec-formatted, e.g. `NoteOn C4 mf`
//!
//! Post-MVP: add `View::PianoRoll` and `View::Notation` as new variants
//! and handle them in [`render_event_row`].
//!
//! # Stop keys
//! Space, q, Esc — sends [`LiveViewConfig::stop_command`] to the conductor
//! and exits.
//!
//! # Architecture
//! See DESIGN.md §Live View Architecture.

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Color, Stylize},
    terminal::{self, ClearType},
};
use futures::StreamExt;
use std::io::{stdout, Write};

use crate::{
    transport::TransportCommand,
    update::{EngineUpdate, MidiUpdate, TransportStatus},
    zmq_client::ZmqClient,
};

/// Which view is active in the live view TUI.
///
/// Tab cycles forward through variants in declaration order.
/// New views should be appended here and handled in [`render_event_row`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// Raw hex + decimal bytes — e.g. `90 3C 5B  /  144 60 91`
    #[default]
    Raw,
    /// Codec-formatted symbolic — e.g. `NoteOn C4 mf`
    Symbolic,
}

impl View {
    fn label(self) -> &'static str {
        match self {
            View::Raw      => "Raw",
            View::Symbolic => "Symbolic",
        }
    }

    fn next(self) -> Self {
        match self {
            View::Raw      => View::Symbolic,
            View::Symbolic => View::Raw,
        }
    }
}

/// Direction of a MIDI event relative to the conductor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventDirection {
    /// Incoming from hardware (monitor, record)
    In,
    /// Outgoing to hardware (playback)
    Out,
    /// Both directions (future: simultaneous play + record)
    Both,
}

/// Configuration for a live view session.
pub struct LiveViewConfig {
    /// Displayed in the TUI header — port name, file name, etc.
    pub label: String,

    /// Which event directions to display and how to colour them.
    ///
    /// At MVP, `In` and `Out` each show all events in a single colour.
    /// `Both` shows incoming in cyan and outgoing in yellow.
    /// Post-MVP: extend with per-channel colour assignment.
    pub direction: EventDirection,

    /// Initial view mode. User can switch with Tab.
    pub initial_view: View,

    /// Command sent to the conductor when the user exits the TUI.
    /// Typically `TransportCommand::Stop`.
    pub stop_command: TransportCommand,

    /// Whether to show transport state (playing/recording, BPM) in the
    /// header. Set `true` for play and record, `false` for monitor-only.
    pub show_transport: bool,
}

/// A recorded event for display in the TUI.
#[derive(Debug, Clone)]
struct DisplayEvent {
    /// Reconstructed status byte (e.g. 0x90 for NoteOn ch0).
    status: u8,
    note:   u8,
    vel:    u8,
    dir:    EventDirection,
}

/// RAII guard — enters alternate screen + raw mode on construction,
/// restores the terminal exactly on drop.
///
/// Using the alternate screen means the reedline prompt, command history,
/// and all previous output are preserved in the primary screen buffer and
/// restored exactly when the guard is dropped — the user sees no visual
/// disruption when the live view exits.
///
/// Drop errors are intentionally ignored: we may already be in an error
/// path and there is nothing useful to do with a secondary terminal failure.
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

/// Run the live view TUI.
///
/// Blocks until the user presses Space, q, or Esc, then sends
/// `config.stop_command` to the conductor and returns.
///
/// # Errors
/// Returns `Err` only if the terminal cannot enter raw mode or the
/// alternate screen — both are unrecoverable for the TUI.
pub async fn run_live_view(
    client: &mut ZmqClient,
    config: LiveViewConfig,
) -> std::io::Result<()> {
    let _guard = TerminalGuard::new()?;

    let mut view = config.initial_view;
    let mut events: Vec<DisplayEvent> = Vec::new();
    let mut transport = TransportStatus {
        is_playing:   false,
        is_recording: false,
        bpm:          120.0,
    };
    let mut key_stream = EventStream::new();

    render_screen(&events, view, &config, &transport)?;

    loop {
        tokio::select! {
            update = client.recv_update() => {
                match update {
                    None => {
                        tracing::warn!("recv_update returned None — breaking live view loop");
                        break;
                    }

                    Some(EngineUpdate::Midi(MidiUpdate::EventDispatched {
                        channel, note, vel
                    })) => {
                        let dir = match config.direction {
                            EventDirection::Both => EventDirection::In,
                            d => d,
                        };
                        let status = 0x90 | (channel & 0x0F);
                        events.push(DisplayEvent { status, note, vel, dir });
                        render_screen(&events, view, &config, &transport)?;
                    }

                    Some(EngineUpdate::Transport(ts)) => {
                        transport = ts;
                        render_screen(&events, view, &config, &transport)?;
                    }

                    Some(_) => {}
                }
            }

            key = key_stream.next() => {
                match key {
                    None => {
                        tracing::warn!("key_stream exhausted — breaking live view loop");
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::warn!(error = %e, "key_stream error");
                    }
                    Some(Ok(Event::Key(KeyEvent { code, modifiers, .. }))) => {
                        match (code, modifiers) {
                            (KeyCode::Char(' '), _)
                            | (KeyCode::Char('q'), _)
                            | (KeyCode::Esc, _)
                            | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,

                            (KeyCode::Tab, _) => {
                                view = view.next();
                                render_screen(&events, view, &config, &transport)?;
                            }

                            _ => {}
                        }
                    }
                    Some(Ok(_)) => {}
                }
            }
        }
    }

    // Always stop the conductor before returning, regardless of exit key.
    let _ = client.send_command(&config.stop_command).await;

    Ok(())
}

/// Render the full TUI screen from scratch.
///
/// Clears the alternate screen and redraws on every event or keypress.
/// Full redraws are acceptable at MIDI event rates (< 1000/s) — a
/// diff-based renderer is not needed at MVP.
fn render_screen(
    events:    &[DisplayEvent],
    view:      View,
    config:    &LiveViewConfig,
    transport: &TransportStatus,
) -> std::io::Result<()> {
    let mut out = stdout();
    let (cols, rows) = terminal::size()?;
    let cols = cols as usize;

    queue!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    // ── Header bar ────────────────────────────────────────────────────────
    let transport_str = if config.show_transport {
        let mode = match (transport.is_playing, transport.is_recording) {
            (true,  true)  => "● recording",
            (true,  false) => "▶ playing",
            (false, _)     => "■ stopped",
        };
        format!("  │  {}  │  {:.1} BPM", mode, transport.bpm)
    } else {
        String::new()
    };

    let header = format!(
        " {}{}  │  Tab: cycle view  │  Space/q/Esc: stop",
        config.label, transport_str
    );
    let header = format!("{:<width$}", header, width = cols);
    queue!(
        out,
        style::PrintStyledContent(header.negative()),
        cursor::MoveToNextLine(1),
    )?;

    // ── View indicator ────────────────────────────────────────────────────
    queue!(
        out,
        style::Print(format!(" View: {}  (Tab to switch)\r\n", view.label())),
        style::Print(format!(" {}\r\n", "─".repeat(cols.saturating_sub(2)))),
    )?;

    // ── Column headers ────────────────────────────────────────────────────
    match view {
        View::Raw => queue!(
            out,
            style::Print(format!(
                " {:<4}  {:<3}  {:<11}  {}\r\n",
                "#", "DIR", "HEX", "DEC"
            )),
        )?,
        View::Symbolic => queue!(
            out,
            style::Print(format!(
                " {:<4}  {:<3}  {:<2}  {:<8}  {:<4}  {}\r\n",
                "#", "DIR", "CH", "TYPE", "VEL", "NOTE"
            )),
        )?,
    }
    queue!(
        out,
        style::Print(format!(" {}\r\n", "─".repeat(cols.saturating_sub(2)))),
    )?;

    // ── Event rows ────────────────────────────────────────────────────────
    // Header occupies 5 lines; status bar occupies 1 line at bottom.
    let max_rows = (rows as usize).saturating_sub(6);
    let skip = events.len().saturating_sub(max_rows);

    for (i, event) in events.iter().enumerate().skip(skip) {
        let (dir_glyph, dir_color) = match event.dir {
            EventDirection::In   => ("←", Color::Cyan),
            EventDirection::Out  => ("→", Color::Yellow),
            EventDirection::Both => ("↔", Color::Green),
        };

        let row = render_event_row(i + 1, event, view, dir_glyph);
        queue!(
            out,
            style::PrintStyledContent(
                style::style(format!(" {}\r\n", row)).with(dir_color)
            ),
        )?;
    }

    // ── Status bar ────────────────────────────────────────────────────────
    queue!(
        out,
        cursor::MoveTo(0, rows - 1),
        style::PrintStyledContent(
            format!(" {} events", events.len()).negative()
        ),
    )?;

    out.flush()
}

/// Format a single event row for the given view.
fn render_event_row(
    idx:       usize,
    event:     &DisplayEvent,
    view:      View,
    dir_glyph: &str,
) -> String {
    match view {
        View::Raw => format!(
            "{:<4}  {:<3}  {:02X} {:02X} {:02X}  {:>3} {:>3} {:>3}",
            idx, dir_glyph,
            event.status, event.note, event.vel,
            event.status, event.note, event.vel,
        ),

        View::Symbolic => {
            let channel = event.status & 0x0F;

            // Decode status nibble to event type name.
            let kind = match event.status & 0xF0 {
                0x80 => "NoteOff",
                0x90 => if event.vel == 0 { "NoteOff" } else { "NoteOn" },
                0xA0 => "PolyAT",
                0xB0 => "CC",
                0xC0 => "ProgramChg",
                0xD0 => "ChanAT",
                0xE0 => "PitchBend",
                _    => "Unknown",
            };

            // Note name from MIDI note number.
            let note_names = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];
            let octave    = (event.note / 12) as i8 - 1;
            let note_name = format!("{}{}", note_names[(event.note % 12) as usize], octave);

            // Velocity as approximate dynamic marking.
            let vel_name = match event.vel {
                0        => "off",
                1..=31   => "pp",
                32..=47  => "p",
                48..=63  => "mp",
                64..=79  => "mf",
                80..=95  => "f",
                96..=111 => "ff",
                _        => "fff",
            };

            format!(
                "{:<4}  {:<3}  {:<2}  {:<8}  {:<4}  {}",
                idx, dir_glyph, channel, kind, vel_name, note_name,
            )
        }
    }
}