//! MIDI input task.
//!
//! Owns a `midir::MidiInputConnection` on midir's callback thread.
//! Forwards every received event to the Conductor via an `mpsc`
//! channel as an [`InputEvent`].
//!
//! The Conductor decides whether to push events into the pre-roll
//! buffer, the record buffer, or neither — this module is policy-free.

use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::Instant;

use midir::MidiInput;
use tokio::sync::mpsc;
use tracing::warn;

use crate::update::MidiUpdate;

/// A single MIDI event as received from the OS input callback.
///
/// Carries both a wall-clock timestamp and the playhead sample
/// position at the moment of receipt.
///
/// - `wall_time`  — ground truth for ordering and autosave.
/// - `sample_pos` — used when mapping events onto the timeline.
///   `0` if the transport is not playing at receipt time.
#[derive(Debug, Clone)]
pub struct InputEvent {
    /// Wall-clock time of receipt.
    pub wall_time:  Instant,
    /// Playhead sample position at receipt.
    pub sample_pos: u64,
    /// Raw MIDI bytes. Always 3 bytes — 2-byte messages pad with 0.
    pub bytes: [u8; 3],
}

/// Owns the active MIDI input connection.
///
/// Dropping this value closes the midir connection and stops the
/// input callback — intentional RAII.
pub struct MidiInputHandle {
    /// Kept alive to hold the connection open.
    /// midir closes the port when this is dropped.
    _connection: midir::MidiInputConnection<()>,
}

impl MidiInputHandle {
    /// Open a MIDI input connection to the named port.
    ///
    /// Every received event is forwarded to `event_tx` as an
    /// [`InputEvent`]. `sample_pos` is populated from `playhead`
    /// at the moment the callback fires.
    ///
    /// # Errors
    /// Returns `Err` if no port with `port_name` exists or the
    /// connection fails.
    pub fn open(
        port_name: &str,
        event_tx:  mpsc::Sender<InputEvent>,
        playhead:  Arc<AtomicU64>,
        status_tx: mpsc::Sender<MidiUpdate>,
    ) -> Result<Self, String> {
        let input = MidiInput::new("mush-midi-in").map_err(|e| e.to_string())?;
        let ports  = input.ports();

        let port = ports
            .iter()
            .find(|p| input.port_name(p).as_deref() == Ok(port_name))
            .ok_or_else(|| format!("MIDI input port '{port_name}' not found"))?;

        let port_name_owned = port_name.to_owned();

        let connection = input
            .connect(
                port,
                "mush-midi-in",
                move |_stamp, raw, _| {
                    if raw.is_empty() || raw.len() > 3 {
                        warn!(len = raw.len(), "midi input: unexpected message length");
                        return;
                    }

                    let mut bytes = [0u8; 3];
                    bytes[..raw.len()].copy_from_slice(raw);

                    let event = InputEvent {
                        wall_time:  Instant::now(),
                        sample_pos: playhead.load(Ordering::Acquire),
                        bytes,
                    };

                    if event_tx.try_send(event).is_err() {
                        warn!("midi input: event_tx full or closed — event dropped");
                    }
                },
                (),
            )
            .map_err(|e| {
                let msg = format!(
                    "failed to connect to MIDI input '{port_name_owned}': {e}"
                );
                let _ = status_tx.try_send(MidiUpdate::Error(msg.clone()));
                msg
            })?;

        Ok(Self { _connection: connection })
    }

    /// Open a connection to the first available MIDI input port.
    ///
    /// Used when no port has been explicitly selected via
    /// `TransportCommand::MidiInputPort`.
    ///
    /// Returns the handle and the name of the port that was opened.
    ///
    /// # Errors
    /// Returns `Err` if no input ports are available or the
    /// connection fails.
    pub fn open_first(
        event_tx:  mpsc::Sender<InputEvent>,
        playhead:  Arc<AtomicU64>,
        status_tx: mpsc::Sender<MidiUpdate>,
    ) -> Result<(Self, String), String> {
        let input = MidiInput::new("mush-midi-in").map_err(|e| e.to_string())?;
        let ports  = input.ports();

        if ports.is_empty() {
            return Err("no MIDI input ports available".into());
        }

        let port      = &ports[0];
        let port_name = input
            .port_name(port)
            .unwrap_or_else(|_| "unknown".into());

        let handle = Self::open(&port_name, event_tx, playhead, status_tx)?;
        Ok((handle, port_name))
    }
}