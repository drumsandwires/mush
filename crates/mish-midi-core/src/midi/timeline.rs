// mush/crates/mish-midi-core/src/midi/timeline.rs
//
// Converts a MIDI file into a sorted, sample-accurate event list
// suitable for linear dispatch from the audio callback.
//
// No I/O, no threads, no hardware. Pure transformation logic.

use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use mush_core::transport::HasSamplePos;
use std::path::Path;

/// A single event on the sample-accurate timeline.
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    /// Absolute position in samples from the start of the project.
    pub sample_pos: u64,
    /// The event payload.
    pub payload: EventPayload,
}

impl HasSamplePos for TimelineEvent {
    fn sample_pos(&self) -> u64 {
        self.sample_pos
    }
}

/// The data carried by a timeline event.
#[derive(Debug, Clone)]
pub enum EventPayload {
    /// A raw MIDI message — channel, note, velocity or equivalent.
    /// Stored as three bytes matching the MIDI wire format.
    /// Two-byte messages (e.g. ProgramChange) use the third byte as 0.
    Midi([u8; 3]),

    /// A tempo change event.
    /// Stored as BPM for display — the timeline builder uses
    /// microseconds_per_beat internally during construction.
    TempoChange(f64),
}

/// Build a sample-accurate timeline from raw MIDI bytes.
///
/// # Arguments
/// - `bytes`       — raw contents of a .mid file
/// - `sample_rate` — audio device sample rate in Hz (e.g. 44_100)
/// - `initial_bpm` — tempo to use before any tempo event is encountered
///
/// # Returns
/// A sorted `Vec<TimelineEvent>` ready for linear dispatch.
pub fn load_midi_bytes(
    bytes: &[u8],
    sample_rate: u32,
    initial_bpm: f64,
) -> Result<Vec<TimelineEvent>, Box<dyn std::error::Error>> {
    let smf = Smf::parse(bytes)?;

    let ticks_per_beat = match smf.header.timing {
        Timing::Metrical(tpb) => tpb.as_int() as u64,
        Timing::Timecode(fps, subframe) => {
            // Convert SMPTE timecode to an equivalent ticks_per_beat.
            // Rare in practice — most MIDI files use Metrical timing.
            (fps.as_f32() * subframe as f32) as u64
        }
    };

    let mut events: Vec<TimelineEvent> = Vec::new();

    // Each track is processed independently.
    // Tempo events from any track affect all subsequent tracks.
    // We collect all events first, then sort by sample_pos.
    let mut tempo_map = TempoMap::new(initial_bpm, ticks_per_beat, sample_rate);

    for track in &smf.tracks {
        let mut absolute_tick: u64 = 0;

        for event in track {
            absolute_tick += event.delta.as_int() as u64;

            match event.kind {
                TrackEventKind::Midi { channel, message } => {
                    let sample_pos = tempo_map.ticks_to_samples(absolute_tick);
                    let payload = midi_message_to_payload(channel.as_int(), message);

                    if let Some(payload) = payload {
                        events.push(TimelineEvent {
                            sample_pos,
                            payload,
                        });
                    }
                }

                TrackEventKind::Meta(MetaMessage::Tempo(us_per_beat)) => {
                    let sample_pos = tempo_map.ticks_to_samples(absolute_tick);
                    let bpm = 60_000_000.0 / us_per_beat.as_int() as f64;

                    // Register the tempo change so subsequent events
                    // use the new tempo for their sample position calculation.
                    tempo_map.add_change(absolute_tick, us_per_beat.as_int() as u64);

                    events.push(TimelineEvent {
                        sample_pos,
                        payload: EventPayload::TempoChange(bpm),
                    });
                }

                // All other meta messages (lyrics, markers, etc.)
                // are ignored at MVP.
                _ => {}
            }
        }
    }

    // Merge all tracks into a single sorted timeline.
    events.sort_by_key(|e| e.sample_pos);

    Ok(events)
}

/// Load a MIDI file from disk and build its timeline.
pub fn load_midi(
    path: &Path,
    sample_rate: u32,
    initial_bpm: f64,
) -> Result<Vec<TimelineEvent>, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    load_midi_bytes(&bytes, sample_rate, initial_bpm)
}

/// Tracks tempo changes and converts tick positions to sample positions.
///
/// A MIDI file can change tempo multiple times. Each segment between
/// tempo changes must be converted using the tempo active during that
/// segment. TempoMap accumulates these segments as changes are
/// encountered during parsing.
struct TempoMap {
    /// Each entry: (absolute_tick, us_per_beat, samples_at_this_tick)
    /// Sorted by absolute_tick ascending.
    changes: Vec<(u64, u64, f64)>,
    ticks_per_beat: u64,
    sample_rate: u32,
}

impl TempoMap {
    fn new(initial_bpm: f64, ticks_per_beat: u64, sample_rate: u32) -> Self {
        let initial_us = (60_000_000.0 / initial_bpm) as u64;
        Self {
            changes: vec![(0, initial_us, 0.0)],
            ticks_per_beat,
            sample_rate,
        }
    }

    /// Register a tempo change at a given absolute tick position.
    fn add_change(&mut self, at_tick: u64, us_per_beat: u64) {
        // Calculate the sample position at the point of this change
        // using the current tempo map state.
        let samples_at_change = self.ticks_to_samples(at_tick);
        self.changes
            .push((at_tick, us_per_beat, samples_at_change as f64));
    }

    /// Convert an absolute tick position to an absolute sample position.
    fn ticks_to_samples(&self, absolute_tick: u64) -> u64 {
        // Find the last tempo change at or before absolute_tick.
        let (base_tick, us_per_beat, base_samples) = self
            .changes
            .iter()
            .rev()
            .find(|(tick, _, _)| *tick <= absolute_tick)
            .copied()
            .unwrap_or((0, 500_000, 0.0));

        let ticks_since_change = absolute_tick - base_tick;
        let samples_per_tick = (us_per_beat as f64 / self.ticks_per_beat as f64)
            * (self.sample_rate as f64 / 1_000_000.0);

        (base_samples + ticks_since_change as f64 * samples_per_tick) as u64
    }
}

/// Convert a midly MidiMessage to an EventPayload.
/// Returns None for messages we do not handle at MVP.
fn midi_message_to_payload(channel: u8, message: MidiMessage) -> Option<EventPayload> {
    let bytes = match message {
        MidiMessage::NoteOn { key, vel } => [0x90 | channel, key.as_int(), vel.as_int()],

        MidiMessage::NoteOff { key, vel } => [0x80 | channel, key.as_int(), vel.as_int()],

        MidiMessage::Controller { controller, value } => {
            [0xB0 | channel, controller.as_int(), value.as_int()]
        }

        MidiMessage::ProgramChange { program } => [0xC0 | channel, program.as_int(), 0],

        MidiMessage::PitchBend { bend } => {
            let value = (bend.as_int() + 8192) as u16;
            let lsb = (value & 0x7F) as u8;
            let msb = ((value >> 7) & 0x7F) as u8;
            [0xE0 | channel, lsb, msb]
        }

        // Aftertouch, SysEx, etc. — deferred to Phase 2
        _ => return None,
    };

    Some(EventPayload::Midi(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MID: &[u8] = include_bytes!("../../../../tests/fixtures/test.mid");

    #[test]
    fn timeline_is_non_empty() {
        let events = load_midi_bytes(TEST_MID, 44_100, 120.0).unwrap();
        assert!(!events.is_empty());
    }

    #[test]
    fn timeline_is_sorted() {
        let events = load_midi_bytes(TEST_MID, 44_100, 120.0).unwrap();
        assert!(
            events
                .windows(2)
                .all(|w| w[0].sample_pos <= w[1].sample_pos),
            "timeline is not sorted by sample_pos"
        );
    }

    #[test]
    fn timeline_contains_midi_events() {
        let events = load_midi_bytes(TEST_MID, 44_100, 120.0).unwrap();
        assert!(
            events
                .iter()
                .any(|e| matches!(e.payload, EventPayload::Midi(_))),
            "no MIDI events found in timeline"
        );
    }

    #[test]
    fn higher_sample_rate_gives_larger_positions() {
        let e44 = load_midi_bytes(TEST_MID, 44_100, 120.0).unwrap();
        let e48 = load_midi_bytes(TEST_MID, 48_000, 120.0).unwrap();
        assert!(
            e48.last().unwrap().sample_pos > e44.last().unwrap().sample_pos,
            "48kHz timeline should have larger sample positions than 44.1kHz"
        );
    }

    #[test]
    fn has_sample_pos_impl_returns_field() {
        let event = TimelineEvent {
            sample_pos: 12_345,
            payload: EventPayload::Midi([0x90, 60, 100]),
        };
        assert_eq!(event.sample_pos(), 12_345);
    }

    #[test]
    fn tempo_map_default_is_120_bpm() {
        let map = TempoMap::new(120.0, 480, 44_100);
        // At 120 BPM, 480 ticks = 1 beat = 0.5 seconds = 22050 samples
        assert_eq!(map.ticks_to_samples(480), 22_050);
    }
    #[test]
    fn tempo_map_140_bpm() {
        let map = TempoMap::new(140.0, 480, 44_100);
        // At 140 BPM, 480 ticks = 1 beat = 60/140 seconds
        let expected = (44_100.0 * 60.0 / 140.0) as u64;
        let actual = map.ticks_to_samples(480);
        // Allow ±1 sample tolerance for floating-point rounding
        assert!(
            actual.abs_diff(expected) <= 1,
            "expected {expected} ±1, got {actual}"
        );
    }
}
