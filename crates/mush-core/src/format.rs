use crate::update::{AudioUpdate, EngineUpdate, MidiUpdate};

/// Convert an [`EngineUpdate`] into a display string.
///
/// Returns `None` for updates that should be silently suppressed —
/// currently only high-frequency [`AudioUpdate::PlayheadMoved`] ticks.
///
/// All three shells (`mish`, `mash`, `mush`) delegate to this function.
/// Shell-specific formatting can be layered on top by the caller if needed
/// in future, but for MVP the output is identical across shells.
pub fn format_update(update: &EngineUpdate) -> Option<String> {
    match update {
        // ── Transport ────────────────────────────────────────────────────
        EngineUpdate::Audio(AudioUpdate::TransportStarted) => {
            Some("▶ transport started".into())
        }

        EngineUpdate::Audio(AudioUpdate::TransportStopped) => {
            Some("■ transport stopped".into())
        }

        // High frequency — suppress entirely.
        EngineUpdate::Audio(AudioUpdate::PlayheadMoved(_)) => None,

        // ── Audio devices ────────────────────────────────────────────────
        EngineUpdate::Audio(AudioUpdate::DeviceList(s)) => {
            Some(format!("audio devices:\n{s}"))
        }

        EngineUpdate::Audio(AudioUpdate::Error(e)) => {
            Some(format!("✗ audio error: {e}"))
        }

        // ── MIDI ─────────────────────────────────────────────────────────
        EngineUpdate::Midi(MidiUpdate::PortList(s)) => {
            Some(format!("MIDI ports:\n{s}"))
        }

        EngineUpdate::Midi(MidiUpdate::EventDispatched { channel, note, vel }) => {
            Some(format!(
                "▶ MIDI event — ch:{channel} note:{note} vel:{vel}"
            ))
        }

        EngineUpdate::Midi(MidiUpdate::Error(e)) => {
            Some(format!("✗ MIDI error: {e}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::{AudioUpdate, EngineUpdate, MidiUpdate};

    #[test]
    fn transport_started_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::TransportStarted);
        assert_eq!(format_update(&u), Some("▶ transport started".into()));
    }

    #[test]
    fn transport_stopped_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::TransportStopped);
        assert_eq!(format_update(&u), Some("■ transport stopped".into()));
    }

    #[test]
    fn playhead_suppressed() {
        let u = EngineUpdate::Audio(AudioUpdate::PlayheadMoved(1024));
        assert_eq!(format_update(&u), None);
    }

    #[test]
    fn device_list_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::DeviceList("ALSA: hw:0".into()));
        assert_eq!(
            format_update(&u),
            Some("audio devices:\nALSA: hw:0".into())
        );
    }

    #[test]
    fn audio_error_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::Error("buffer underrun".into()));
        assert_eq!(
            format_update(&u),
            Some("✗ audio error: buffer underrun".into())
        );
    }

    #[test]
    fn port_list_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::PortList("port 0: IAC Driver".into()));
        assert_eq!(
            format_update(&u),
            Some("MIDI ports:\nport 0: IAC Driver".into())
        );
    }

    #[test]
    fn midi_error_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::Error("no device".into()));
        assert_eq!(
            format_update(&u),
            Some("✗ MIDI error: no device".into())
        );
    }

    #[test]
    fn event_dispatched_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::EventDispatched {
            channel: 1,
            note: 60,
            vel: 100,
        });
        assert_eq!(
            format_update(&u),
            Some("▶ MIDI event — ch:1 note:60 vel:100".into())
        );
    }
}