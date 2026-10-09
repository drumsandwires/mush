// crates/mish-midi-core/src/midi/engine.rs  (REPLACE)
use crate::midi::display::list_enumerated_ports;
use crate::midi::enumerate::enumerate_ports;
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
    Ports,
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
pub struct MidiEngine {
    /// Refreshable cache of enumerated MIDI ports.
    /// None = unpopulated or invalidated; Some = ready to serve.
    devices_cache: RwLock<Option<Result<Vec<MidiPortInfo>, String>>>,
}

impl MidiEngine {
    pub fn new() -> Self {
        Self {
            devices_cache: RwLock::new(None),
        }
    }

    /// Invalidates the devices cache so the next call to `ports()`
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

    /// Lists available MIDI ports.
    /// Serves from cache when populated; enumerates from OS on cache miss.
    /// Refresh with `invalidate_devices_cache()` before calling if needed.
    pub fn ports(&self) -> MidiResponse {
        // --- fast path: cache hit ---
        {
            let read = self.devices_cache.read().unwrap();
            if let Some(ref cached) = *read {
                return Self::format_ports(cached);
            }
        } // read lock dropped here before write below

        // --- slow path: cache miss — enumerate and populate ---
        let result = enumerate_ports().map_err(|e| e.to_string());
        let response = Self::format_ports(&result);
        *self.devices_cache.write().unwrap() = Some(result);
        response
    }

    /// Refreshes the devices cache unconditionally, then returns the listing.
    /// Use this when you know the port topology may have changed.
    pub fn ports_refresh(&self) -> MidiResponse {
        self.invalidate_devices_cache();
        self.ports()
    }

    pub fn play(&self) -> MidiResponse {
        // TODO: trigger real MIDI playback
        MidiResponse::Output("Playing...".to_string())
    }

    pub fn stop(&self) -> MidiResponse {
        // TODO: stop real MIDI playback
        MidiResponse::Output("Stopped.".to_string())
    }

    pub fn load(&self, path: &Path) -> MidiResponse {
        // TODO: load MIDI file
        MidiResponse::Output(format!("Loading '{}'...", path.display()))
    }

    pub fn save(&self, path: &Path) -> MidiResponse {
        // TODO: save MIDI file
        MidiResponse::Output(format!("Saving to '{}'...", path.display()))
    }

    pub fn record(&self) -> MidiResponse {
        // TODO: start MIDI recording
        MidiResponse::Output("Recording...".to_string())
    }

    fn format_ports(result: &Result<Vec<MidiPortInfo>, String>) -> MidiResponse {
        match result {
            Err(e) => MidiResponse::Output(format!("error enumerating ports: {e}")),
            Ok(ports) => match list_enumerated_ports(ports) {
                Some(listing) => MidiResponse::Output(listing),
                None => MidiResponse::Output("no MIDI ports found".to_string()),
            },
        }
    }
}

impl Default for MidiEngine {
    fn default() -> Self {
        Self::new()
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
    fn test_ports_returns_output() {
        // Content is hardware-dependent; assert only on shape.
        assert!(matches!(MidiEngine::new().ports(), MidiResponse::Output(_)));
    }

    #[test]
    fn test_ports_refresh() {
        let engine = MidiEngine::new();
        let _ = engine.ports();
        assert!(matches!(engine.ports_refresh(), MidiResponse::Output(_)));
    }

    #[test]
    fn test_invalidate_then_ports() {
        let engine = MidiEngine::new();
        let _ = engine.ports();
        engine.invalidate_devices_cache();
        assert!(matches!(engine.ports(), MidiResponse::Output(_)));
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
