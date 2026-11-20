//! Generates MIDI test fixtures for the mush test suite.
//!
//! Run with:
//!   cargo run -p mush-core --bin gen_fixtures -- <subcommand>
//!
//! Subcommands:
//!   all         — generate all fixtures (default)
//!   scale       — C major scale, 8 quarter notes, 120 BPM (4 bars)
//!   dense       — 128 notes at every tick, for MidiQueueFull testing
//!   long        — 32-bar C major scale loop, for sustained playback testing

use midly::{Format, Header, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};
use std::path::{Path, PathBuf};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| "all".to_string());

    let out_dir = Path::new("tests/fixtures");
    std::fs::create_dir_all(out_dir).unwrap();

    match arg.as_str() {
        "all" => {
            write_fixture(out_dir.join("test.mid"), scale());
            write_fixture(out_dir.join("dense.mid"), dense());
            write_fixture(out_dir.join("long.mid"), long());
        }
        "scale" => write_fixture(out_dir.join("test.mid"), scale()),
        "dense" => write_fixture(out_dir.join("dense.mid"), dense()),
        "long" => write_fixture(out_dir.join("long.mid"), long()),
        other => {
            eprintln!("unknown subcommand: {other}");
            eprintln!("usage: gen_fixtures [all|scale|dense|long]");
            std::process::exit(1);
        }
    }
}

// ── Shared header ─────────────────────────────────────────────────────────────

fn header() -> Header {
    Header {
        format: Format::SingleTrack,
        timing: Timing::Metrical(480.into()),
    }
}

/// Tempo meta-event: 120 BPM = 500_000 µs/beat.
fn tempo_event() -> TrackEvent<'static> {
    TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(midly::MetaMessage::Tempo(500_000.into())),
    }
}

fn note_on(delta: u32, note: u8, vel: u8) -> TrackEvent<'static> {
    TrackEvent {
        delta: delta.into(),
        kind: TrackEventKind::Midi {
            channel: 0.into(),
            message: MidiMessage::NoteOn {
                key: note.into(),
                vel: vel.into(),
            },
        },
    }
}

fn note_off(delta: u32, note: u8) -> TrackEvent<'static> {
    TrackEvent {
        delta: delta.into(),
        kind: TrackEventKind::Midi {
            channel: 0.into(),
            message: MidiMessage::NoteOff {
                key: note.into(),
                vel: 0.into(),
            },
        },
    }
}

fn end_of_track() -> TrackEvent<'static> {
    TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
    }
}

// ── Fixtures ──────────────────────────────────────────────────────────────────

/// C major scale — 8 quarter notes at 120 BPM.
/// Used as the general-purpose baseline fixture (test.mid).
/// ST-3-1, ST-3-2, ST-3-3, ST-4-1.
fn scale() -> Smf<'static> {
    const NOTES: &[u8] = &[60, 62, 64, 65, 67, 69, 71, 72]; // C4–C5
    const QUARTER: u32 = 480;

    let mut track: Track = vec![tempo_event()];

    for &note in NOTES {
        track.push(note_on(0, note, 100));
        track.push(note_off(QUARTER, note));
    }

    track.push(end_of_track());
    Smf {
        header: header(),
        tracks: vec![track],
    }
}

/// Dense fixture — 128 consecutive notes with delta=1 tick between each.
///
/// Produces a burst of events that arrive faster than the audio callback
/// can drain them at normal queue capacity. Used to trigger and verify
/// `DiagEvent::MidiQueueFull` (ST-4-3).
fn dense() -> Smf<'static> {
    let mut track: Track = vec![tempo_event()];

    for note in 0u8..=127 {
        track.push(note_on(0, note, 100));
        track.push(note_off(1, note));
    }

    track.push(end_of_track());
    Smf {
        header: header(),
        tracks: vec![track],
    }
}

/// Long fixture — C major scale repeated 32 times (~32 bars at 120 BPM).
///
/// Provides ~64 seconds of continuous playback for sustained RT diagnostic
/// observation and port-disconnect testing (ST-4-1, ST-4-2).
fn long() -> Smf<'static> {
    const NOTES: &[u8] = &[60, 62, 64, 65, 67, 69, 71, 72];
    const QUARTER: u32 = 480;

    let mut track: Track = vec![tempo_event()];

    for _ in 0..32 {
        for &note in NOTES {
            track.push(note_on(0, note, 100));
            track.push(note_off(QUARTER, note));
        }
    }

    track.push(end_of_track());
    Smf {
        header: header(),
        tracks: vec![track],
    }
}

// ── I/O ───────────────────────────────────────────────────────────────────────

fn write_fixture(path: PathBuf, smf: Smf) {
    let mut buf = Vec::new();
    smf.write_std(&mut buf).expect("MIDI serialisation failed");
    std::fs::write(&path, &buf).unwrap();
    println!("written {} bytes → {}", buf.len(), path.display());
}
