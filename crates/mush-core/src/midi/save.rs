//! Timeline serialisation — `.mid` and `.mudl` export.
//!
//! Entry point: [`save_timeline`].
//!
//! # Strategy
//! Both formats share the same first step: reconstruct a [`midly::Smf`]
//! from the in-memory timeline. For `.mid` the Smf is serialised
//! directly via midly. For `.mudl` it is passed to
//! [`mudld::midi_to_kdl`] and the resulting KDL document is written.
//!
//! # Tick reconstruction
//! The timeline stores sample positions, not ticks. We reverse the
//! conversion using the current BPM and the original tick resolution
//! preserved from load time. Tempo change events in the timeline are
//! used to reconstruct the tempo map, so multi-tempo files round-trip
//! correctly.

use std::path::Path;

use anyhow::{Context, bail};
use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind,
    num::{u4, u7, u15, u24, u28},
};
use tracing::debug;

use crate::midi::timeline::MidiPayload;
use crate::timeline::TimelineEvent;

/// Save the timeline to `path`.
///
/// The output format is determined by the file extension:
/// - `.mid`  — standard MIDI file (format 0, single track)
/// - `.mudl` — MUDL KDL document via `mudld::midi_to_kdl`
///
/// `ticks_per_beat` is the tick resolution preserved from the last
/// `LoadMidi` / `LoadMudl` call, defaulting to 480 if no file has
/// been loaded.
///
/// # Errors
/// Returns `Err` for unknown extensions, serialisation failures,
/// or I/O errors.
pub fn save_timeline(
    timeline: &[TimelineEvent<MidiPayload>],
    path: &Path,
    bpm: f64,
    sample_rate: u32,
    ticks_per_beat: u16,
) -> anyhow::Result<()> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "mid" | "midi" => {
            let smf = timeline_to_smf(timeline, bpm, sample_rate, ticks_per_beat)?;
            write_mid(&smf, path)
        }
        "mudl" => {
            let smf = timeline_to_smf(timeline, bpm, sample_rate, ticks_per_beat)?;
            write_mudl(&smf, path)
        }
        other => bail!("unknown save format '.{other}' — use .mid or .mudl"),
    }
}

/// Reconstruct a [`midly::Smf`] from the in-memory timeline.
///
/// Produces a format-0 (single track) SMF. Tempo changes in the
/// timeline are emitted as `MetaMessage::Tempo` events so that
/// multi-tempo files round-trip correctly.
///
/// Sample positions are converted back to ticks using the tempo map
/// reconstructed from `TempoChange` payloads in the timeline.
fn timeline_to_smf<'a>(
    timeline: &[TimelineEvent<MidiPayload>],
    initial_bpm: f64,
    sample_rate: u32,
    ticks_per_beat: u16,
) -> anyhow::Result<Smf<'a>> {
    // Build a reverse tempo map: sample_pos → us_per_beat.
    // Each TempoChange event in the timeline marks a new segment.
    let mut tempo_segments: Vec<(u64, u64)> = vec![(0, bpm_to_us(initial_bpm))];

    for event in timeline {
        if let MidiPayload::TempoChange(bpm) = event.payload {
            tempo_segments.push((event.sample_pos, bpm_to_us(bpm)));
        }
    }

    // Convert a sample position to absolute ticks using the tempo map.
    let sample_to_tick = |sample_pos: u64| -> u64 {
        // Find the tempo segment active at sample_pos.
        let seg_idx = tempo_segments
            .partition_point(|(seg_sample, _)| *seg_sample <= sample_pos)
            .saturating_sub(1);

        let (seg_sample, us_per_beat) = tempo_segments[seg_idx];

        let ticks_before: u64 = {
            let mut acc = 0u64;
            for i in 0..seg_idx {
                let (s0, us0) = tempo_segments[i];
                let s1 = tempo_segments
                    .get(i + 1)
                    .map(|(s, _)| *s)
                    .unwrap_or(sample_pos);
                acc += samples_to_ticks(s1 - s0, us0, sample_rate, ticks_per_beat);
            }
            acc
        };

        ticks_before
            + samples_to_ticks(
                sample_pos - seg_sample,
                us_per_beat,
                sample_rate,
                ticks_per_beat,
            )
    };

    // Build the single track.
    let mut track_events: Vec<(u64, TrackEventKind<'a>)> = Vec::new();

    // Initial tempo event at tick 0.
    let initial_us = bpm_to_us(initial_bpm);
    track_events.push((
        0,
        TrackEventKind::Meta(MetaMessage::Tempo(u24::from(initial_us as u32))),
    ));

    for event in timeline {
        let abs_tick = sample_to_tick(event.sample_pos);

        match event.payload {
            MidiPayload::TempoChange(bpm) => {
                let us = bpm_to_us(bpm);
                track_events.push((
                    abs_tick,
                    TrackEventKind::Meta(MetaMessage::Tempo(u24::from(us as u32))),
                ));
            }
            MidiPayload::Midi(bytes) => {
                let status = bytes[0] & 0xF0;
                let channel = u4::from(bytes[0] & 0x0F);
                let b1 = u7::from(bytes[1] & 0x7F);
                let b2 = u7::from(bytes[2] & 0x7F);

                let message = match status {
                    0x80 => MidiMessage::NoteOff { key: b1, vel: b2 },
                    0x90 => MidiMessage::NoteOn { key: b1, vel: b2 },
                    0xB0 => MidiMessage::Controller {
                        controller: b1,
                        value: b2,
                    },
                    0xC0 => MidiMessage::ProgramChange { program: b1 },
                    0xE0 => {
                        // Reconstruct 14-bit pitch bend value.
                        let raw = (bytes[1] as i16) | ((bytes[2] as i16) << 7);
                        let centered = raw - 8192;
                        MidiMessage::PitchBend {
                            bend: midly::PitchBend::from_int(centered),
                        }
                    }
                    _ => continue, // skip unrecognised status bytes
                };

                track_events.push((abs_tick, TrackEventKind::Midi { channel, message }));
            }
        }
    }

    // Sort by absolute tick — timeline should already be sorted but
    // be defensive in case QueueEvent interleaved events.
    track_events.sort_by_key(|(tick, _)| *tick);

    // Convert absolute ticks to delta ticks.
    let mut prev_tick = 0u64;
    let mut track: Track<'a> = Track::new();

    for (abs_tick, kind) in track_events {
        let delta = abs_tick.saturating_sub(prev_tick);
        prev_tick = abs_tick;
        track.push(TrackEvent {
            delta: u28::from(delta as u32),
            kind,
        });
    }

    // End-of-track.
    track.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let header = Header {
        format: Format::SingleTrack,
        timing: Timing::Metrical(u15::from(ticks_per_beat)),
    };

    Ok(Smf {
        header,
        tracks: vec![track],
    })
}

/// Write an [`Smf`] to a `.mid` file.
fn write_mid(smf: &Smf<'_>, path: &Path) -> anyhow::Result<()> {
    let mut buf = Vec::new();
    smf.write_std(&mut buf)
        .with_context(|| format!("failed to serialise SMF for {}", path.display()))?;
    std::fs::write(path, &buf).with_context(|| format!("failed to write {}", path.display()))?;
    debug!(path = %path.display(), bytes = buf.len(), "saved .mid");
    Ok(())
}

/// Write an [`Smf`] to a `.mudl` file via `mudld::midi_to_kdl`.
fn write_mudl(smf: &Smf<'_>, path: &Path) -> anyhow::Result<()> {
    let doc = mudld::midi_to_kdl(smf);
    std::fs::write(path, doc.to_string())
        .with_context(|| format!("failed to write {}", path.display()))?;
    debug!(path = %path.display(), "saved .mudl");
    Ok(())
}

/// Convert BPM to microseconds per beat.
#[inline]
fn bpm_to_us(bpm: f64) -> u64 {
    (60_000_000.0 / bpm) as u64
}

/// Convert a sample-count duration to ticks given a tempo and resolution.
#[inline]
fn samples_to_ticks(samples: u64, us_per_beat: u64, sample_rate: u32, ticks_per_beat: u16) -> u64 {
    if us_per_beat == 0 {
        return 0;
    }
    let ticks_per_sample =
        ticks_per_beat as f64 / (us_per_beat as f64 / 1_000_000.0 * sample_rate as f64);
    (samples as f64 * ticks_per_sample) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi::timeline::load_midi_bytes;
    use std::path::PathBuf;

    const TEST_MID: &[u8] = include_bytes!("../../../../tests/fixtures/test.mid");
    const SAMPLE_RATE: u32 = 44_100;
    const INITIAL_BPM: f64 = 120.0;

    #[test]
    fn bpm_to_us_120() {
        assert_eq!(bpm_to_us(120.0), 500_000);
    }

    #[test]
    fn bpm_to_us_140() {
        // 60_000_000 / 140 = 428_571 (truncated)
        assert_eq!(bpm_to_us(140.0), 428_571);
    }

    #[test]
    fn samples_to_ticks_roundtrip() {
        // At 120 BPM, 480 tpb, 44100 sr:
        // 1 beat = 22050 samples = 480 ticks
        let ticks = samples_to_ticks(22_050, 500_000, 44_100, 480);
        assert!(ticks.abs_diff(480) <= 1, "expected 480 ±1, got {ticks}");
    }

    #[test]
    fn unknown_extension_returns_error() {
        let timeline = vec![];
        let path = PathBuf::from("output.wav");
        let result = save_timeline(&timeline, &path, 120.0, SAMPLE_RATE, 480);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("unknown save format")
        );
    }

    #[test]
    fn save_empty_timeline_mid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.mid");
        let result = save_timeline(&[], &path, 120.0, SAMPLE_RATE, 480);
        assert!(result.is_ok(), "{:?}", result);
        assert!(path.exists());
    }

    #[test]
    fn save_empty_timeline_mudl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.mudl");
        let result = save_timeline(&[], &path, 120.0, SAMPLE_RATE, 480);
        assert!(result.is_ok(), "{:?}", result);
        assert!(path.exists());
    }

    #[test]
    fn save_mid_produces_parseable_file() {
        let (timeline, tpb) = load_midi_bytes(TEST_MID, SAMPLE_RATE, INITIAL_BPM).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.mid");
        save_timeline(&timeline, &path, INITIAL_BPM, SAMPLE_RATE, tpb).unwrap();

        // Round-trip: the saved file must parse without error.
        let bytes = std::fs::read(&path).unwrap();
        let parsed = Smf::parse(&bytes);
        assert!(parsed.is_ok(), "saved .mid did not parse: {:?}", parsed);
    }

    #[test]
    fn save_mudl_produces_parseable_kdl() {
        let (timeline, tpb) = load_midi_bytes(TEST_MID, SAMPLE_RATE, INITIAL_BPM).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.mudl");
        save_timeline(&timeline, &path, INITIAL_BPM, SAMPLE_RATE, tpb).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let parsed: Result<kdl::KdlDocument, _> = contents.parse();
        assert!(parsed.is_ok(), "saved .mudl is not valid KDL: {:?}", parsed);
    }

    #[test]
    fn save_mid_event_count_preserved() {
        let (timeline, tpb) = load_midi_bytes(TEST_MID, SAMPLE_RATE, INITIAL_BPM).unwrap();
        let midi_event_count = timeline
            .iter()
            .filter(|e| matches!(e.payload, MidiPayload::Midi(_)))
            .count();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.mid");
        save_timeline(&timeline, &path, INITIAL_BPM, SAMPLE_RATE, tpb).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        let smf = Smf::parse(&bytes).unwrap();
        let saved_count = smf
            .tracks
            .iter()
            .flat_map(|t| t.iter())
            .filter(|e| matches!(e.kind, TrackEventKind::Midi { .. }))
            .count();

        assert_eq!(
            saved_count, midi_event_count,
            "event count changed after round-trip: {midi_event_count} → {saved_count}"
        );
    }
}

