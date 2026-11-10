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
        EngineUpdate::Audio(AudioUpdate::TransportStarted) => {
            Some("▶ transport started".into())
        }
        EngineUpdate::Audio(AudioUpdate::TransportStopped) => {
            Some("■ transport stopped".into())
        }
        EngineUpdate::Audio(AudioUpdate::PlayheadMoved(_)) => None,
        EngineUpdate::Audio(AudioUpdate::DeviceList(s)) => {
            Some(format!("audio devices:\n{s}"))
        }
        EngineUpdate::Audio(AudioUpdate::HostList(hosts)) => {
            if hosts.is_empty() {
                Some("audio hosts: none found".into())
            } else {
                Some(format!("audio hosts:\n{}", hosts.join("\n")))
            }
        }
        EngineUpdate::Audio(AudioUpdate::ActiveHost(name)) => Some(match name {
            Some(n) => format!("audio host: {n}"),
            None    => "audio host: (default)".into(),
        }),
        EngineUpdate::Audio(AudioUpdate::ActiveDevice(name)) => Some(match name {
            Some(n) => format!("audio device: {n}"),
            None    => "audio device: (default)".into(),
        }),
        EngineUpdate::Audio(AudioUpdate::Error(e)) => {
            Some(format!("✗ audio error: {e}"))
        }
        EngineUpdate::Midi(MidiUpdate::PortList(s)) => {
            Some(format!("MIDI ports:\n{s}"))
        }
        EngineUpdate::Midi(MidiUpdate::EventDispatched { channel, note, vel }) => {
            Some(format!("▶ MIDI event — ch:{channel} note:{note} vel:{vel}"))
        }
        EngineUpdate::Midi(MidiUpdate::ActivePort(name)) => Some(match name {
            Some(n) => format!("MIDI port: {n}"),
            None    => "MIDI port: (none selected)".into(),
        }),
        EngineUpdate::Midi(MidiUpdate::LoadMudlAck(path)) => {
            Some(format!("✓ loaded {path}"))
        }
        EngineUpdate::Midi(MidiUpdate::SaveAck(path)) => {
            Some(format!("✓ saved {path}"))
        }
        EngineUpdate::Midi(MidiUpdate::RecordSaved(path)) => {
            Some(format!("✓ recording saved to {path}"))
        }
        EngineUpdate::Midi(MidiUpdate::RecordEmpty) => {
            Some("recording stopped — no events captured".into())
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

    // ── AudioUpdate ───────────────────────────────────────────────────────────

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
        assert_eq!(format_update(&u), Some("audio devices:\nALSA: hw:0".into()));
    }

    #[test]
    fn host_list_empty_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::HostList(vec![]));
        assert_eq!(format_update(&u), Some("audio hosts: none found".into()));
    }

    #[test]
    fn host_list_single_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::HostList(vec!["ALSA".into()]));
        assert_eq!(format_update(&u), Some("audio hosts:\nALSA".into()));
    }

    #[test]
    fn host_list_multiple_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::HostList(vec![
            "ALSA".into(),
            "JACK".into(),
        ]));
        assert_eq!(
            format_update(&u),
            Some("audio hosts:\nALSA\nJACK".into())
        );
    }

    #[test]
    fn active_host_some_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::ActiveHost(Some("JACK".into())));
        assert_eq!(format_update(&u), Some("audio host: JACK".into()));
    }

    #[test]
    fn active_host_none_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::ActiveHost(None));
        assert_eq!(format_update(&u), Some("audio host: (default)".into()));
    }

    #[test]
    fn active_device_some_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::ActiveDevice(Some("hw:0".into())));
        assert_eq!(format_update(&u), Some("audio device: hw:0".into()));
    }

    #[test]
    fn active_device_none_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::ActiveDevice(None));
        assert_eq!(format_update(&u), Some("audio device: (default)".into()));
    }

    #[test]
    fn audio_error_formats() {
        let u = EngineUpdate::Audio(AudioUpdate::Error("buffer underrun".into()));
        assert_eq!(
            format_update(&u),
            Some("✗ audio error: buffer underrun".into())
        );
    }

    // ── MidiUpdate ────────────────────────────────────────────────────────────

    #[test]
    fn port_list_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::PortList("port 0: IAC Driver".into()));
        assert_eq!(
            format_update(&u),
            Some("MIDI ports:\nport 0: IAC Driver".into())
        );
    }

    #[test]
    fn event_dispatched_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::EventDispatched {
            channel: 1,
            note:    60,
            vel:     100,
        });
        assert_eq!(
            format_update(&u),
            Some("▶ MIDI event — ch:1 note:60 vel:100".into())
        );
    }

    #[test]
    fn active_port_some_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::ActivePort(Some("IAC Driver Bus 1".into())));
        assert_eq!(
            format_update(&u),
            Some("MIDI port: IAC Driver Bus 1".into())
        );
    }

    #[test]
    fn active_port_none_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::ActivePort(None));
        assert_eq!(
            format_update(&u),
            Some("MIDI port: (none selected)".into())
        );
    }

    #[test]
    fn load_mudl_ack_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::LoadMudlAck("song.mudl".into()));
        assert_eq!(format_update(&u), Some("✓ loaded song.mudl".into()));
    }

    #[test]
    fn save_ack_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::SaveAck("output/song.mid".into()));
        assert_eq!(
            format_update(&u),
            Some("✓ saved output/song.mid".into())
        );
    }

    #[test]
    fn record_saved_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::RecordSaved("/tmp/take-1.mudl".into()));
        assert_eq!(
            format_update(&u),
            Some("✓ recording saved to /tmp/take-1.mudl".into())
        );
    }

    #[test]
    fn record_empty_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::RecordEmpty);
        assert_eq!(
            format_update(&u),
            Some("recording stopped — no events captured".into())
        );
    }

    #[test]
    fn midi_error_formats() {
        let u = EngineUpdate::Midi(MidiUpdate::Error("no device".into()));
        assert_eq!(format_update(&u), Some("✗ MIDI error: no device".into()));
    }
}