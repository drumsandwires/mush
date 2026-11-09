//! Pre-roll buffer — always-on capture of incoming MIDI events.
//!
//! Stores [`InputEvent`]s in a bounded `VecDeque`. When the buffer
//! reaches [`PRE_ROLL_CAPACITY`] the oldest events are drained and
//! autosaved to a `.mudl` file before new events are appended.
//!
//! # Why `Mutex<VecDeque>` rather than `ArrayQueue`?
//! The pre-roll needs to be drained on autosave and record-stop.
//! `ArrayQueue` is optimised for single-element push/pop on the RT
//! path. `VecDeque` behind a `Mutex` is appropriate here — the
//! pre-roll is written from the Conductor's async task, not from
//! the audio callback.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use anyhow::Context;
use tracing::{info, warn};

use crate::midi::input::InputEvent;

/// Maximum number of events the pre-roll buffer holds before autosave.
///
/// TODO: make this configurable via `ConductorConfig` post-MVP.
pub const PRE_ROLL_CAPACITY: usize = 65_536;

/// The pre-roll buffer.
///
/// Written by the Conductor's async task when Listen or Record is
/// active. Drained on autosave wrap and record-stop.
pub struct PreRoll {
    events: Mutex<VecDeque<InputEvent>>,
    autosave_path: Mutex<Option<PathBuf>>,
}

impl PreRoll {
    /// Create a new, empty pre-roll buffer.
    pub fn new() -> Self {
        Self {
            events: Mutex::new(VecDeque::with_capacity(PRE_ROLL_CAPACITY)),
            autosave_path: Mutex::new(None),
        }
    }

    /// Set the autosave path.
    /// Pass `None` to revert to the timestamped default.
    pub fn set_autosave_path(&self, path: Option<PathBuf>) {
        *self.autosave_path.lock().unwrap() = path;
    }

    /// Push an event into the buffer.
    ///
    /// If the buffer is at capacity, drains all events and autosaves
    /// them before pushing the new event.
    pub fn push(&self, event: InputEvent) {
        let mut events = self.events.lock().unwrap();

        if events.len() >= PRE_ROLL_CAPACITY {
            let drained: Vec<InputEvent> = events.drain(..).collect();
            drop(events); // release lock before I/O

            let path = self.next_autosave_path();
            match save_events(&drained, &path) {
                Ok(()) => info!(
                    path = %path.display(),
                    events = drained.len(),
                    "pre-roll autosaved"
                ),
                Err(e) => warn!(
                    path = %path.display(),
                    error = %e,
                    "pre-roll autosave failed"
                ),
            }

            self.events.lock().unwrap().push_back(event);
        } else {
            events.push_back(event);
        }
    }

    /// Snapshot the current contents without draining.
    ///
    /// Used by `Record { start: true }` to mark the capture
    /// start point without disturbing the rolling buffer.
    pub fn snapshot(&self) -> Vec<InputEvent> {
        self.events.lock().unwrap().iter().cloned().collect()
    }

    /// Drain all events from the buffer and return them.
    pub fn drain(&self) -> Vec<InputEvent> {
        self.events.lock().unwrap().drain(..).collect()
    }

    /// Current number of events in the buffer.
    pub fn len(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    /// True if the buffer contains no events.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the configured autosave path, or generate a
    /// timestamped default in the current working directory.
    pub fn next_autosave_path(&self) -> PathBuf {
        if let Some(ref path) = *self.autosave_path.lock().unwrap() {
            return path.clone();
        }
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        PathBuf::from(format!("mush-preroll-{ts}.mudl"))
    }
}

impl Default for PreRoll {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialise a slice of [`InputEvent`]s to a `.mudl` file.
///
/// Events are written as a single-track MUDL document. The
/// wall-clock timestamp of the first event is the time origin;
/// subsequent events are stored as millisecond offsets in the
/// `delta` property. `sample-pos` is stored as a property for
/// future timeline mapping.
///
/// TODO: use `sample_pos` for timeline mapping when sample rate
/// is known at save time.
pub fn save_events(events: &[InputEvent], path: &Path) -> anyhow::Result<()> {
    use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

    if events.is_empty() {
        return Ok(());
    }

    let origin = events[0].wall_time;

    let mut root = KdlDocument::new();
    let mut midi_node = KdlNode::new("midi");
    midi_node.push(KdlEntry::new_prop("format", KdlValue::Integer(0)));
    midi_node.push(KdlEntry::new_prop("ticks-per-beat", KdlValue::Integer(480)));

    let mut track_node = KdlNode::new("track");
    track_node.push(KdlEntry::new_prop("index", KdlValue::Integer(0)));

    let mut events_doc = KdlDocument::new();

    for event in events {
        let offset_ms = event
            .wall_time
            .saturating_duration_since(origin)
            .as_millis() as i128;

        let status = event.bytes[0] & 0xF0;
        let channel = (event.bytes[0] & 0x0F) as i128;

        let msg_node = match status {
            0x80 => {
                let mut n = KdlNode::new("note-off");
                n.push(KdlEntry::new_prop("channel", KdlValue::Integer(channel)));
                n.push(KdlEntry::new_prop(
                    "key",
                    KdlValue::Integer(event.bytes[1] as i128),
                ));
                n.push(KdlEntry::new_prop(
                    "vel",
                    KdlValue::Integer(event.bytes[2] as i128),
                ));
                n
            }
            0x90 => {
                let mut n = KdlNode::new("note-on");
                n.push(KdlEntry::new_prop("channel", KdlValue::Integer(channel)));
                n.push(KdlEntry::new_prop(
                    "key",
                    KdlValue::Integer(event.bytes[1] as i128),
                ));
                n.push(KdlEntry::new_prop(
                    "vel",
                    KdlValue::Integer(event.bytes[2] as i128),
                ));
                n
            }
            0xA0 => {
                let mut n = KdlNode::new("aftertouch");
                n.push(KdlEntry::new_prop("channel", KdlValue::Integer(channel)));
                n.push(KdlEntry::new_prop(
                    "key",
                    KdlValue::Integer(event.bytes[1] as i128),
                ));
                n.push(KdlEntry::new_prop(
                    "vel",
                    KdlValue::Integer(event.bytes[2] as i128),
                ));
                n
            }
            0xB0 => {
                let mut n = KdlNode::new("controller");
                n.push(KdlEntry::new_prop("channel", KdlValue::Integer(channel)));
                n.push(KdlEntry::new_prop(
                    "cc",
                    KdlValue::Integer(event.bytes[1] as i128),
                ));
                n.push(KdlEntry::new_prop(
                    "value",
                    KdlValue::Integer(event.bytes[2] as i128),
                ));
                n
            }
            0xC0 => {
                let mut n = KdlNode::new("program-change");
                n.push(KdlEntry::new_prop("channel", KdlValue::Integer(channel)));
                n.push(KdlEntry::new_prop(
                    "program",
                    KdlValue::Integer(event.bytes[1] as i128),
                ));
                n
            }
            0xD0 => {
                let mut n = KdlNode::new("channel-aftertouch");
                n.push(KdlEntry::new_prop("channel", KdlValue::Integer(channel)));
                n.push(KdlEntry::new_prop(
                    "vel",
                    KdlValue::Integer(event.bytes[1] as i128),
                ));
                n
            }
            0xE0 => {
                let bend = (event.bytes[1] as i128) | ((event.bytes[2] as i128) << 7);
                let mut n = KdlNode::new("pitch-bend");
                n.push(KdlEntry::new_prop("channel", KdlValue::Integer(channel)));
                n.push(KdlEntry::new_prop("value", KdlValue::Integer(bend)));
                n
            }
            _ => {
                let mut n = KdlNode::new("meta-other");
                n.push(KdlEntry::new_prop(
                    "type",
                    KdlValue::Integer(event.bytes[0] as i128),
                ));
                n.push(KdlEntry::new(KdlValue::String(format!(
                    "{:02x} {:02x}",
                    event.bytes[1], event.bytes[2]
                ))));
                n
            }
        };

        let mut child_doc = KdlDocument::new();
        child_doc.nodes_mut().push(msg_node);

        let mut ev = KdlNode::new("event");
        ev.push(KdlEntry::new_prop("delta", KdlValue::Integer(offset_ms)));
        ev.push(KdlEntry::new_prop(
            "sample-pos",
            KdlValue::Integer(event.sample_pos as i128),
        ));
        ev.set_children(child_doc);

        events_doc.nodes_mut().push(ev);
    }

    // MIDI spec: every track ends with end-of-track.
    let mut eot_child = KdlDocument::new();
    eot_child.nodes_mut().push(KdlNode::new("end-of-track"));
    let mut eot = KdlNode::new("event");
    eot.push(KdlEntry::new_prop("delta", KdlValue::Integer(0)));
    eot.set_children(eot_child);
    events_doc.nodes_mut().push(eot);

    track_node.set_children(events_doc);

    let mut track_doc = KdlDocument::new();
    track_doc.nodes_mut().push(track_node);
    midi_node.set_children(track_doc);
    root.nodes_mut().push(midi_node);

    std::fs::write(path, root.to_string())
        .with_context(|| format!("failed to write pre-roll to {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn make_event(bytes: [u8; 3]) -> InputEvent {
        InputEvent {
            wall_time: Instant::now(),
            sample_pos: 0,
            bytes,
        }
    }

    #[test]
    fn push_and_len() {
        let pr = PreRoll::new();
        pr.push(make_event([0x90, 60, 100]));
        pr.push(make_event([0x80, 60, 0]));
        assert_eq!(pr.len(), 2);
    }

    #[test]
    fn drain_empties_buffer() {
        let pr = PreRoll::new();
        pr.push(make_event([0x90, 60, 100]));
        let drained = pr.drain();
        assert_eq!(drained.len(), 1);
        assert!(pr.is_empty());
    }

    #[test]
    fn snapshot_does_not_drain() {
        let pr = PreRoll::new();
        pr.push(make_event([0x90, 60, 100]));
        let snap = pr.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(pr.len(), 1);
    }

    #[test]
    fn set_autosave_path_overrides_default() {
        let pr = PreRoll::new();
        let path = PathBuf::from("/tmp/test-preroll.mudl");
        pr.set_autosave_path(Some(path.clone()));
        assert_eq!(pr.next_autosave_path(), path);
    }

    #[test]
    fn default_autosave_path_is_timestamped() {
        let pr = PreRoll::new();
        let path = pr.next_autosave_path();
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(name.starts_with("mush-preroll-"));
        assert!(name.ends_with(".mudl"));
    }

    #[test]
    fn save_events_writes_valid_kdl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.mudl");
        let events = vec![make_event([0x90, 60, 100]), make_event([0x80, 60, 0])];
        save_events(&events, &path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("note-on"));
        assert!(contents.contains("note-off"));
    }

    #[test]
    fn save_events_empty_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.mudl");
        save_events(&[], &path).unwrap();
        assert!(!path.exists());
    }
}
