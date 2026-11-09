use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use midly::{MidiMessage, Smf, TrackEventKind};

/// Convert a parsed [`midly::Smf`] to a [`kdl::KdlDocument`].
///
/// The returned document can be serialised with `doc.to_string()`.
pub fn midi_to_kdl(smf: &Smf<'_>) -> KdlDocument {
    let mut root       = KdlDocument::new();
    let mut midi_node  = KdlNode::new("midi");

    let format_num: i128 = match smf.header.format {
        midly::Format::SingleTrack => 0,
        midly::Format::Parallel    => 1,
        midly::Format::Sequential  => 2,
    };
    midi_node.push(KdlEntry::new_prop("format", KdlValue::Integer(format_num)));

    match smf.header.timing {
        midly::Timing::Metrical(tpb) => {
            midi_node.push(KdlEntry::new_prop(
                "ticks-per-beat",
                KdlValue::Integer(u16::from(tpb) as i128),
            ));
        }
        midly::Timing::Timecode(fps, sub) => {
            midi_node.push(KdlEntry::new_prop(
                "fps",
                KdlValue::Integer(fps.as_int() as i128),
            ));
            midi_node.push(KdlEntry::new_prop(
                "subframes",
                KdlValue::Integer(sub as i128),
            ));
        }
    }

    let mut tracks_doc = KdlDocument::new();

    for (idx, track) in smf.tracks.iter().enumerate() {
        let mut track_node = KdlNode::new("track");
        track_node.push(KdlEntry::new_prop("index", KdlValue::Integer(idx as i128)));

        let mut events_doc = KdlDocument::new();

        for event in track {
            let mut ev = KdlNode::new("event");
            ev.push(KdlEntry::new_prop(
                "delta",
                KdlValue::Integer(u32::from(event.delta) as i128),
            ));

            let mut child_doc = KdlDocument::new();
            child_doc.nodes_mut().push(build_event_node(&event.kind));
            ev.set_children(child_doc);

            events_doc.nodes_mut().push(ev);
        }

        track_node.set_children(events_doc);
        tracks_doc.nodes_mut().push(track_node);
    }

    midi_node.set_children(tracks_doc);
    root.nodes_mut().push(midi_node);
    root
}

// ── event node builder ────────────────────────────────────────────────────────

fn build_event_node(kind: &TrackEventKind<'_>) -> KdlNode {
    match kind {
        TrackEventKind::Midi { channel, message } => {
            let ch = u8::from(*channel) as i128;
            match message {
                MidiMessage::NoteOn { key, vel } => midi_msg(
                    "note-on", ch,
                    &[("key", u8::from(*key) as i128), ("vel", u8::from(*vel) as i128)],
                ),
                MidiMessage::NoteOff { key, vel } => midi_msg(
                    "note-off", ch,
                    &[("key", u8::from(*key) as i128), ("vel", u8::from(*vel) as i128)],
                ),
                MidiMessage::ProgramChange { program } => midi_msg(
                    "program-change", ch,
                    &[("program", u8::from(*program) as i128)],
                ),
                MidiMessage::Controller { controller, value } => midi_msg(
                    "controller", ch,
                    &[
                        ("cc",    u8::from(*controller) as i128),
                        ("value", u8::from(*value)      as i128),
                    ],
                ),
                MidiMessage::PitchBend { bend } => midi_msg(
                    "pitch-bend", ch,
                    &[("value", bend.as_int() as i128)],
                ),
                MidiMessage::Aftertouch { key, vel } => midi_msg(
                    "aftertouch", ch,
                    &[("key", u8::from(*key) as i128), ("vel", u8::from(*vel) as i128)],
                ),
                MidiMessage::ChannelAftertouch { vel } => midi_msg(
                    "channel-aftertouch", ch,
                    &[("vel", u8::from(*vel) as i128)],
                ),
            }
        }

        TrackEventKind::Meta(meta) => {
            use midly::MetaMessage;
            match meta {
                MetaMessage::TrackNumber(n) => {
                    let mut node = KdlNode::new("track-number");
                    if let Some(n) = n {
                        node.push(KdlEntry::new(KdlValue::Integer(*n as i128)));
                    }
                    node
                }
                MetaMessage::Text(data)           => text_meta_node("text",            data),
                MetaMessage::Copyright(data)      => text_meta_node("copyright",       data),
                MetaMessage::TrackName(data)      => text_meta_node("track-name",      data),
                MetaMessage::InstrumentName(data) => text_meta_node("instrument-name", data),
                MetaMessage::Lyric(data)          => text_meta_node("lyric",           data),
                MetaMessage::Marker(data)         => text_meta_node("marker",          data),
                MetaMessage::CuePoint(data)       => text_meta_node("cue-point",       data),
                MetaMessage::ProgramName(data)    => text_meta_node("program-name",    data),
                MetaMessage::DeviceName(data)     => text_meta_node("device-name",     data),
                MetaMessage::MidiChannel(ch) => prop_node(
                    "midi-channel",
                    &[("channel", u8::from(*ch) as i128)],
                ),
                MetaMessage::MidiPort(port) => prop_node(
                    "midi-port",
                    &[("port", u8::from(*port) as i128)],
                ),
                MetaMessage::EndOfTrack => KdlNode::new("end-of-track"),
                MetaMessage::Tempo(us)  => prop_node(
                    "tempo",
                    &[("us-per-beat", u32::from(*us) as i128)],
                ),
                MetaMessage::SmpteOffset(s) => {
                    let mut n = KdlNode::new("smpte-offset");
                    n.push(KdlEntry::new_prop("fps",      KdlValue::Integer(s.fps().as_int() as i128)));
                    n.push(KdlEntry::new_prop("hour",     KdlValue::Integer(s.hour()     as i128)));
                    n.push(KdlEntry::new_prop("minute",   KdlValue::Integer(s.minute()   as i128)));
                    n.push(KdlEntry::new_prop("second",   KdlValue::Integer(s.second()   as i128)));
                    n.push(KdlEntry::new_prop("frame",    KdlValue::Integer(s.frame()    as i128)));
                    n.push(KdlEntry::new_prop("subframe", KdlValue::Integer(s.subframe() as i128)));
                    n
                }
                MetaMessage::TimeSignature(num, den_pow, clocks, notes) => prop_node(
                    "time-signature",
                    &[
                        ("num",    *num    as i128),
                        ("den",    2_i128.pow(*den_pow as u32)),
                        ("clocks", *clocks as i128),
                        ("notes",  *notes  as i128),
                    ],
                ),
                MetaMessage::KeySignature(key, minor) => {
                    let mut n = prop_node("key-signature", &[("key", *key as i128)]);
                    n.push(KdlEntry::new_prop("minor", KdlValue::Bool(*minor)));
                    n
                }
                MetaMessage::SequencerSpecific(data) => {
                    let mut n = KdlNode::new("sequencer-specific");
                    n.push(KdlEntry::new(KdlValue::String(hex_encode(data))));
                    n
                }
                MetaMessage::Unknown(type_byte, data) => {
                    let mut n = KdlNode::new("meta-other");
                    n.push(KdlEntry::new_prop("type", KdlValue::Integer(*type_byte as i128)));
                    n.push(KdlEntry::new(KdlValue::String(hex_encode(data))));
                    n
                }
            }
        }

        TrackEventKind::SysEx(data) => {
            let mut n = KdlNode::new("sysex");
            n.push(KdlEntry::new(KdlValue::String(hex_encode(data))));
            n
        }

        TrackEventKind::Escape(data) => {
            let mut n = KdlNode::new("escape");
            n.push(KdlEntry::new(KdlValue::String(hex_encode(data))));
            n
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn midi_msg(name: &str, channel: i128, extra: &[(&str, i128)]) -> KdlNode {
    let mut n = KdlNode::new(name);
    n.push(KdlEntry::new_prop("channel", KdlValue::Integer(channel)));
    for (k, v) in extra {
        n.push(KdlEntry::new_prop(*k, KdlValue::Integer(*v)));
    }
    n
}

fn prop_node(name: &str, props: &[(&str, i128)]) -> KdlNode {
    let mut n = KdlNode::new(name);
    for (k, v) in props {
        n.push(KdlEntry::new_prop(*k, KdlValue::Integer(*v)));
    }
    n
}

/// Valid UTF-8 → string value. Non-UTF-8 → hex string + `hex=true` property.
fn text_meta_node(node_name: &str, data: &[u8]) -> KdlNode {
    let mut n = KdlNode::new(node_name);
    match std::str::from_utf8(data) {
        Ok(s)  => n.push(KdlEntry::new(KdlValue::String(s.to_string()))),
        Err(_) => {
            n.push(KdlEntry::new_prop("hex", KdlValue::Bool(true)));
            n.push(KdlEntry::new(KdlValue::String(hex_encode(data))));
        }
    }
    n
}

fn hex_encode(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}