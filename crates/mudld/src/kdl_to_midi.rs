//! KDL → MIDI conversion.
//!
//! # Memory note
//! [`leak_bytes`] promotes heap allocations to `'static` so that
//! `midly::Smf<'static>` can be returned without lifetime entanglement.
//!
//! TODO: replace `leak_bytes` with a `bumpalo` arena so long-running
//! processes (e.g. `mush-core`) do not accumulate leaked allocations.

use anyhow::{bail, Context, Result};
use kdl::{KdlDocument, KdlNode};
use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind,
    num::{u4, u7, u15, u24, u28},
};

/// Convert a parsed [`kdl::KdlDocument`] to a [`midly::Smf`].
///
/// Returns `Smf<'static>` — byte slices are promoted via [`leak_bytes`].
///
/// # Errors
/// Returns `Err` if the document is structurally invalid or contains
/// unknown event node names.
pub fn kdl_to_smf(doc: &KdlDocument) -> Result<Smf<'static>> {
    let midi_node = doc
        .get("midi")
        .context("KDL document must have a top-level `midi` node")?;

    let format = match get_i128(midi_node, "format").unwrap_or(1) {
        0 => Format::SingleTrack,
        1 => Format::Parallel,
        2 => Format::Sequential,
        n => bail!("Unknown MIDI format {n}"),
    };

    let timing = if let Some(tpb) = get_i128(midi_node, "ticks-per-beat") {
        Timing::Metrical(u15::from(tpb as u16))
    } else {
        let fps = get_i128(midi_node, "fps").context("Expected ticks-per-beat or fps")?;
        let sub = get_i128(midi_node, "subframes").context("Expected subframes alongside fps")?;
        let fps_val = parse_fps(fps)?;
        Timing::Timecode(fps_val, sub as u8)
    };

    let header   = Header { format, timing };
    let children = midi_node.children().context("midi node has no children")?;

    let mut tracks: Vec<Track<'static>> = Vec::new();

    for track_node in children.nodes() {
        if track_node.name().value() != "track" {
            continue;
        }
        tracks.push(parse_track(track_node)?);
    }

    Ok(Smf { header, tracks })
}

// ── track ─────────────────────────────────────────────────────────────────────

fn parse_track(track_node: &KdlNode) -> Result<Track<'static>> {
    let children = track_node
        .children()
        .context("track node has no children")?;

    let mut events: Track<'static> = Vec::new();
    let mut has_eot = false;

    for event_node in children.nodes() {
        if event_node.name().value() != "event" {
            continue;
        }

        let delta = u28::from(get_i128(event_node, "delta").unwrap_or(0) as u32);

        let msg_doc  = event_node.children().context("event node has no children")?;
        let msg_node = msg_doc.nodes().first().context("event node has no message child")?;

        let kind = parse_event_kind(msg_node)?;
        if matches!(kind, TrackEventKind::Meta(MetaMessage::EndOfTrack)) {
            has_eot = true;
        }

        events.push(TrackEvent { delta, kind });
    }

    // MIDI spec requires every track to end with end-of-track.
    if !has_eot {
        events.push(TrackEvent {
            delta: u28::from(0u32),
            kind:  TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });
    }

    Ok(events)
}

// ── event kind ────────────────────────────────────────────────────────────────

fn parse_event_kind(node: &KdlNode) -> Result<TrackEventKind<'static>> {
    let name = node.name().value();

    // ── channel messages ──────────────────────────────────────────────────────
    match name {
        "note-on" => return Ok(TrackEventKind::Midi {
            channel: u4::from(get_i128(node, "channel").unwrap_or(0) as u8),
            message: MidiMessage::NoteOn {
                key: u7::from(get_i128(node, "key").context("note-on missing key")? as u8),
                vel: u7::from(get_i128(node, "vel").context("note-on missing vel")? as u8),
            },
        }),
        "note-off" => return Ok(TrackEventKind::Midi {
            channel: u4::from(get_i128(node, "channel").unwrap_or(0) as u8),
            message: MidiMessage::NoteOff {
                key: u7::from(get_i128(node, "key").context("note-off missing key")? as u8),
                vel: u7::from(get_i128(node, "vel").context("note-off missing vel")? as u8),
            },
        }),
        "program-change" => return Ok(TrackEventKind::Midi {
            channel: u4::from(get_i128(node, "channel").unwrap_or(0) as u8),
            message: MidiMessage::ProgramChange {
                program: u7::from(
                    get_i128(node, "program").context("program-change missing program")? as u8,
                ),
            },
        }),
        "controller" => return Ok(TrackEventKind::Midi {
            channel: u4::from(get_i128(node, "channel").unwrap_or(0) as u8),
            message: MidiMessage::Controller {
                controller: u7::from(
                    get_i128(node, "cc").context("controller missing cc")? as u8,
                ),
                value: u7::from(
                    get_i128(node, "value").context("controller missing value")? as u8,
                ),
            },
        }),
        "pitch-bend" => return Ok(TrackEventKind::Midi {
            channel: u4::from(get_i128(node, "channel").unwrap_or(0) as u8),
            message: MidiMessage::PitchBend {
                bend: midly::PitchBend::from_int(
                    get_i128(node, "value").context("pitch-bend missing value")? as i16,
                ),
            },
        }),
        "aftertouch" => return Ok(TrackEventKind::Midi {
            channel: u4::from(get_i128(node, "channel").unwrap_or(0) as u8),
            message: MidiMessage::Aftertouch {
                key: u7::from(get_i128(node, "key").context("aftertouch missing key")? as u8),
                vel: u7::from(get_i128(node, "vel").context("aftertouch missing vel")? as u8),
            },
        }),
        "channel-aftertouch" => return Ok(TrackEventKind::Midi {
            channel: u4::from(get_i128(node, "channel").unwrap_or(0) as u8),
            message: MidiMessage::ChannelAftertouch {
                vel: u7::from(
                    get_i128(node, "vel").context("channel-aftertouch missing vel")? as u8,
                ),
            },
        }),
        _ => {}
    }

    // ── meta messages ─────────────────────────────────────────────────────────
    match name {
        "track-number" => {
            let n = node.get(0).and_then(|v| v.as_integer()).map(|n| n as u16);
            return Ok(TrackEventKind::Meta(MetaMessage::TrackNumber(n)));
        }
        "text"            => return text_meta(node, MetaMessage::Text),
        "copyright"       => return text_meta(node, MetaMessage::Copyright),
        "track-name"      => return text_meta(node, MetaMessage::TrackName),
        "instrument-name" => return text_meta(node, MetaMessage::InstrumentName),
        "lyric"           => return text_meta(node, MetaMessage::Lyric),
        "marker"          => return text_meta(node, MetaMessage::Marker),
        "cue-point"       => return text_meta(node, MetaMessage::CuePoint),
        "program-name"    => return text_meta(node, MetaMessage::ProgramName),
        "device-name"     => return text_meta(node, MetaMessage::DeviceName),
        "midi-channel" => {
            let ch = get_i128(node, "channel").context("midi-channel missing channel")? as u8;
            return Ok(TrackEventKind::Meta(MetaMessage::MidiChannel(u4::from(ch))));
        }
        "midi-port" => {
            let port = get_i128(node, "port").context("midi-port missing port")? as u8;
            return Ok(TrackEventKind::Meta(MetaMessage::MidiPort(u7::from(port))));
        }
        "end-of-track" => return Ok(TrackEventKind::Meta(MetaMessage::EndOfTrack)),
        "tempo" => {
            return Ok(TrackEventKind::Meta(MetaMessage::Tempo(u24::from(
                get_i128(node, "us-per-beat").context("tempo missing us-per-beat")? as u32,
            ))));
        }
        "smpte-offset" => {
            let fps      = parse_fps(get_i128(node, "fps").context("smpte-offset missing fps")?)?;
            let hour     = get_i128(node, "hour").unwrap_or(0)     as u8;
            let minute   = get_i128(node, "minute").unwrap_or(0)   as u8;
            let second   = get_i128(node, "second").unwrap_or(0)   as u8;
            let frame    = get_i128(node, "frame").unwrap_or(0)    as u8;
            let subframe = get_i128(node, "subframe").unwrap_or(0) as u8;
            let s = midly::SmpteTime::new(hour, minute, second, frame, subframe, fps)
                .context("smpte-offset values out of range")?;
            return Ok(TrackEventKind::Meta(MetaMessage::SmpteOffset(s)));
        }
        "time-signature" => {
            let num     = get_i128(node, "num").context("time-signature missing num")? as u8;
            let den     = get_i128(node, "den").context("time-signature missing den")? as u32;
            let den_pow = den.trailing_zeros() as u8;
            let clocks  = get_i128(node, "clocks").unwrap_or(24) as u8;
            let notes   = get_i128(node, "notes").unwrap_or(8)   as u8;
            return Ok(TrackEventKind::Meta(MetaMessage::TimeSignature(
                num, den_pow, clocks, notes,
            )));
        }
        "key-signature" => {
            let key   = get_i128(node, "key").context("key-signature missing key")? as i8;
            let minor = get_bool(node, "minor").unwrap_or(false);
            return Ok(TrackEventKind::Meta(MetaMessage::KeySignature(key, minor)));
        }
        "sequencer-specific" => {
            let hex  = get_str(node).context("sequencer-specific missing hex payload")?;
            let data = leak_bytes(&hex_decode(hex)?);
            return Ok(TrackEventKind::Meta(MetaMessage::SequencerSpecific(data)));
        }
        "meta-other" => {
            let type_byte = get_i128(node, "type").context("meta-other missing type")? as u8;
            let hex       = get_str(node).context("meta-other missing hex payload")?;
            let data      = leak_bytes(&hex_decode(hex)?);
            return Ok(TrackEventKind::Meta(MetaMessage::Unknown(type_byte, data)));
        }
        _ => {}
    }

    // ── sysex / escape ────────────────────────────────────────────────────────
    match name {
        "sysex" => {
            let hex  = get_str(node).context("sysex missing hex string")?;
            let data = leak_bytes(&hex_decode(hex)?);
            return Ok(TrackEventKind::SysEx(data));
        }
        "escape" => {
            let hex  = get_str(node).context("escape missing hex string")?;
            let data = leak_bytes(&hex_decode(hex)?);
            return Ok(TrackEventKind::Escape(data));
        }
        _ => {}
    }

    bail!("Unknown event node `{name}` — cannot reconstruct.")
}

// ── KDL accessors ─────────────────────────────────────────────────────────────

fn get_i128(node: &KdlNode, key: &str) -> Option<i128> {
    node.get(key)?.as_integer()
}

fn get_bool(node: &KdlNode, key: &str) -> Option<bool> {
    node.get(key)?.as_bool()
}

/// First positional (non-property) string argument of a node.
fn get_str<'a>(node: &'a KdlNode) -> Option<&'a str> {
    node.get(0)?.as_string()
}

// ── byte / text helpers ───────────────────────────────────────────────────────

/// Decode a text meta node.
/// Handles both plain UTF-8 and hex-encoded fallback (`hex=true`).
fn text_meta<'a, F>(node: &KdlNode, wrap: F) -> Result<TrackEventKind<'static>>
where
    F: Fn(&'static [u8]) -> MetaMessage<'static>,
{
    let is_hex = get_bool(node, "hex").unwrap_or(false);
    let s      = get_str(node).unwrap_or("");
    let bytes  = if is_hex { hex_decode(s)? } else { s.as_bytes().to_vec() };
    Ok(TrackEventKind::Meta(wrap(leak_bytes(&bytes))))
}

/// Promote a heap allocation to `'static`.
///
/// Acceptable in short-lived CLI tools and bounded file-load operations.
///
/// TODO: replace with a `bumpalo` arena allocator so long-running
/// processes do not accumulate leaked allocations per file load.
fn leak_bytes(data: &[u8]) -> &'static [u8] {
    Box::leak(data.to_vec().into_boxed_slice())
}

fn hex_decode(s: &str) -> Result<Vec<u8>> {
    s.split_whitespace()
        .map(|tok| {
            u8::from_str_radix(tok, 16)
                .with_context(|| format!("Bad hex byte `{tok}`"))
        })
        .collect()
}

fn parse_fps(fps: i128) -> Result<midly::Fps> {
    match fps {
        24 => Ok(midly::Fps::Fps24),
        25 => Ok(midly::Fps::Fps25),
        29 => Ok(midly::Fps::Fps29),
        30 => Ok(midly::Fps::Fps30),
        n  => bail!("Invalid MIDI timecode fps: {n} (must be 24, 25, 29, or 30)"),
    }
}