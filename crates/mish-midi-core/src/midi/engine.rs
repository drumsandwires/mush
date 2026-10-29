use crossbeam::queue::ArrayQueue;
use midir::MidiOutput as MidirOutput;
use std::path::Path;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

use crate::midi::display::list_enumerated_devices;
use crate::midi::enumerate::enumerate_devices;
use crate::midi::types::MidiPortInfo;
use mush_core::midi_output::MidiOutput;
use mush_core::rt_priority;
use mush_core::update::MidiUpdate;

/// The result of evaluating a MIDI command.
/// Deliberately decoupled from any REPL or plugin response type —
/// each frontend maps this into its own output representation.
#[derive(Debug, PartialEq)]
pub enum MidiResponse {
    Output(String),
    Empty,
    Exit(String),
}

/// MIDI command evaluator.
/// Pure domain logic — no string parsing, no clap, no shlex, no REPL types.
/// Each frontend (REPL, nushell plugin, GUI) calls methods directly.
#[derive(Default)]
pub struct MidiEngine {
    /// Refreshable cache of enumerated MIDI devices.
    /// None = unpopulated or invalidated; Some = ready to serve.
    devices_cache: RwLock<Option<Result<Vec<MidiPortInfo>, String>>>,
}

impl MidiEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Invalidates the devices cache so the next call to `devices()`
    /// re-enumerates from the OS. Call this before any command that
    /// may change the port topology (e.g. connect, disconnect).
    pub fn invalidate_devices_cache(&self) {
        *self.devices_cache.write().unwrap() = None;
    }

    /// Returns a formatted list of all available MIDI tasks.
    /// Frontends can call this for their own `help` command.
    pub fn help(&self) -> MidiResponse {
        MidiResponse::Output(format!("MIDI commands: {}", "TBD"))
    }

    pub fn version(&self) -> MidiResponse {
        MidiResponse::Output("mish-midi-core v0.1.0".to_string())
    }

    /// Lists available MIDI devices.
    /// Serves from cache when populated; enumerates from OS on cache miss.
    /// Refresh with `invalidate_devices_cache()` before calling if needed.
    pub fn devices(&self) -> MidiResponse {
        // fast path: cache hit
        {
            // TODO(#17): Remove unwrap on cache read.
            let read = self.devices_cache.read().unwrap();
            if let Some(ref cached) = *read {
                return Self::format_devices(cached);
            }
        } // read lock dropped here before write below

        // slow path: cache miss — enumerate and populate
        let result = enumerate_devices().map_err(|e| e.to_string());
        let response = Self::format_devices(&result);
        *self.devices_cache.write().unwrap() = Some(result);
        response
    }

    /// Refreshes the devices cache unconditionally, then returns the listing.
    /// Use this when you know the port topology may have changed.
    pub fn devices_refresh(&self) -> MidiResponse {
        self.invalidate_devices_cache();
        self.devices()
    }

    pub fn play(&self) -> MidiResponse {
        // TODO(#18): Playback MIDI data.
        MidiResponse::Output("TBD: Playing...".to_string())
    }

    pub fn stop(&self) -> MidiResponse {
        // TODO(#19): Stop MIDI playback/recording.
        MidiResponse::Output("TBD: Stopped.".to_string())
    }

    pub fn load(&self, path: &Path) -> MidiResponse {
        // TODO(#20): Load a MIDI file.
        MidiResponse::Output(format!("TBD: Loading '{}'...", path.display()))
    }

    pub fn save(&self, path: &Path) -> MidiResponse {
        // TODO(#21): Save MIDI data to file.
        MidiResponse::Output(format!("TBD: Saving to '{}'...", path.display()))
    }

    pub fn record(&self) -> MidiResponse {
        // TODO(#22): Record MIDI data.
        MidiResponse::Output("TBD: Recording...".to_string())
    }

    pub fn monitor(&self) -> MidiResponse {
        // TODO(#22): Monitor MIDI data.
        MidiResponse::Output("TBD: Monitoring...".to_string())
    }

    fn format_devices(result: &Result<Vec<MidiPortInfo>, String>) -> MidiResponse {
        match result {
            Err(e) => MidiResponse::Output(format!("error enumerating devices: {e}")),
            Ok(devices) => match list_enumerated_devices(devices) {
                Some(listing) => MidiResponse::Output(listing),
                None => MidiResponse::Output("no MIDI devices found".to_string()),
            },
        }
    }
}

// ── DesktopMidiOutput ────────────────────────────────────────────────────────

/// A MIDI output connection backed by midir.
///
/// Implements the `MidiOutput` trait so `MidiEngineRt` does not
/// depend directly on midir — a different backend (e.g. Web MIDI)
/// can be substituted without changing the RT engine.
pub struct DesktopMidiOutput {
    connection: midir::MidiOutputConnection,
}

impl DesktopMidiOutput {
    /// Open a connection to the first available MIDI output port.
    /// Returns `Err` if no ports are available or the connection fails.
    pub fn open_first_port() -> Result<Self, String> {
        let output = MidirOutput::new("mush-midi-out").map_err(|e| e.to_string())?;

        let ports = output.ports();

        if ports.is_empty() {
            return Err("no MIDI output ports available".into());
        }

        let port = &ports[0];
        let name = output.port_name(port).unwrap_or_else(|_| "unknown".into());

        let connection = output.connect(port, &name).map_err(|e| e.to_string())?;

        Ok(Self { connection })
    }
}

impl MidiOutput for DesktopMidiOutput {
    /// Send raw MIDI bytes to the output port.
    /// Called from the MidiEngineRt thread — must not block.
    fn send(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.connection.send(bytes).map_err(|e| e.to_string())
    }
}

// ── MidiEngineRt ─────────────────────────────────────────────────────────────

/// The real-time MIDI dispatch engine.
///
/// Runs on a dedicated high-priority thread.
/// Pops [u8; 3] messages from a lock-free queue filled by the
/// audio callback and sends them to the MIDI output.
///
/// Distinct from `MidiEngine` (the command evaluator):
///
///   MidiEngine   — called by shells and plugins, handles commands
///   MidiEngineRt — driven by the audio callback, dispatches bytes
pub struct MidiEngineRt;

impl MidiEngineRt {
    /// Spawn the RT MIDI dispatch thread.
    ///
    /// # Arguments
    /// - `output`    — MIDI output connection (e.g. `DesktopMidiOutput`)
    /// - `queue`     — lock-free queue filled by the audio callback
    /// - `status_tx` — sends `MidiUpdate` events to the `EngineMonitor`
    /// - `running`   — set to `false` to stop the thread cleanly
    ///
    /// # Returns
    /// A `JoinHandle` — keep it alive for the duration of the program.
    /// Dropping it detaches the thread; the `running` flag stops it.
    pub fn spawn<O: MidiOutput>(
        output: O,
        queue: Arc<ArrayQueue<[u8; 3]>>,
        status_tx: std::sync::mpsc::Sender<MidiUpdate>,
        running: Arc<AtomicBool>,
    ) -> thread::JoinHandle<()> {
        thread::Builder::new()
            .name("mush-midi-rt".into())
            .spawn(move || {
                rt_priority::elevate_current_thread();
                Self::run_loop(output, queue, status_tx, running);
            })
            .expect("failed to spawn MIDI RT thread")
    }

    fn run_loop<O: MidiOutput>(
        mut output: O,
        queue: Arc<ArrayQueue<[u8; 3]>>,
        status_tx: std::sync::mpsc::Sender<MidiUpdate>,
        running: Arc<AtomicBool>,
    ) {
        while running.load(Ordering::Acquire) {
            match queue.pop() {
                Some(bytes) => {
                    // ProgramChange (0xCn) and ChannelPressure (0xDn)
                    // are 2-byte messages — the third byte is padding.
                    let status = bytes[0] & 0xF0;
                    let channel = bytes[0] & 0x0F;
                    let slice = if status == 0xC0 || status == 0xD0 {
                        &bytes[..2]
                    } else {
                        &bytes[..3]
                    };

                    if let Err(e) = output.send(slice) {
                        let _ = status_tx.send(MidiUpdate::Error(e));
                        break;
                    }

                    let _ = status_tx.send(MidiUpdate::EventDispatched {
                        channel,
                        note: bytes[1],
                        vel: bytes[2],
                    });
                }

                None => {
                    // Queue empty — yield rather than spin at 100% CPU.
                    // Returns quickly — does not sleep.
                    thread::yield_now();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help() {
        assert!(matches!(MidiEngine::new().help(), MidiResponse::Output(_)));
    }

    #[test]
    fn test_version() {
        assert!(matches!(
            MidiEngine::new().version(),
            MidiResponse::Output(_)
        ));
    }

    #[test]
    fn test_devices_returns_output() {
        // Content is hardware-dependent; assert only on shape.
        assert!(matches!(
            MidiEngine::new().devices(),
            MidiResponse::Output(_)
        ));
    }

    #[test]
    fn test_devices_refresh() {
        let engine = MidiEngine::new();
        let _ = engine.devices();
        assert!(matches!(engine.devices_refresh(), MidiResponse::Output(_)));
    }

    #[test]
    fn test_invalidate_then_devices() {
        let engine = MidiEngine::new();
        let _ = engine.devices();
        engine.invalidate_devices_cache();
        assert!(matches!(engine.devices(), MidiResponse::Output(_)));
    }

    #[test]
    fn test_play() {
        assert!(matches!(MidiEngine::new().play(), MidiResponse::Output(_)));
    }

    #[test]
    fn test_stop() {
        assert!(matches!(MidiEngine::new().stop(), MidiResponse::Output(_)));
    }

    #[test]
    fn test_load() {
        assert!(matches!(
            MidiEngine::new().load(Path::new("test.mid")),
            MidiResponse::Output(_)
        ));
    }
}
