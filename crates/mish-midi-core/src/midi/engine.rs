use crate::midi::display::list_enumerated_devices;
use crate::midi::enumerate::enumerate_devices;
use crate::midi::types::MidiPortInfo;
use enum_iterator::{Sequence, all};
use std::fmt;
use std::path::Path;
use std::sync::RwLock;

/// The result of evaluating a MIDI command.
/// Deliberately decoupled from any REPL or plugin response type —
/// each frontend maps this into its own output representation.
#[derive(Debug, PartialEq)]
pub enum MidiResponse {
    Output(String),
    Empty,
    Exit(String),
}

/// Canonical list of MIDI tasks — drives help output in any frontend.
#[derive(Debug, PartialEq, Sequence)]
pub enum MidiTasks {
    Help,
    Version,
    Devices,
    Load,
    Save,
    Play,
    Record,
    Stop,
}

impl fmt::Display for MidiTasks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
        MidiResponse::Output(format!(
            "MIDI commands: {}",
            all::<MidiTasks>()
                .map(|t| t.to_string().to_lowercase())
                .collect::<Vec<_>>()
                .join(", ")
        ))
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
