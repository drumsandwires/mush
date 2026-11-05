use crate::transport::HasSamplePos;

/// A single event on the sample-accurate timeline.
///
/// Generic over the payload type `P` so the same timeline
/// machinery serves both MIDI (`MidiPayload`) and audio
/// (`AudioPayload`) without duplication.
///
/// Events are sorted by `sample_pos` ascending after loading.
/// The clock reads them linearly via `next_event_idx`.
///
/// # Type parameter
/// `P` must be `Clone + Send + 'static` — the timeline is shared
/// across threads via `Arc<RwLock<Vec<TimelineEvent<P>>>>`.
#[derive(Debug, Clone)]
pub struct TimelineEvent<P>
where
    P: Clone + Send + 'static,
{
    /// Absolute position in samples from the start of the project.
    pub sample_pos: u64,

    /// The event payload — domain-specific data.
    pub payload: P,
}

impl<P> HasSamplePos for TimelineEvent<P>
where
    P: Clone + Send + 'static,
{
    fn sample_pos(&self) -> u64 {
        self.sample_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal payload for testing — carries no domain data.
    #[derive(Debug, Clone)]
    struct TestPayload;

    #[test]
    fn sample_pos_returned_correctly() {
        let event = TimelineEvent {
            sample_pos: 44_100,
            payload: TestPayload,
        };
        assert_eq!(event.sample_pos(), 44_100);
    }

    #[test]
    fn timeline_sorts_by_sample_pos() {
        let mut events = vec![
            TimelineEvent { sample_pos: 88_200, payload: TestPayload },
            TimelineEvent { sample_pos: 0,      payload: TestPayload },
            TimelineEvent { sample_pos: 44_100, payload: TestPayload },
        ];
        events.sort_by_key(|e| e.sample_pos);
        assert_eq!(events[0].sample_pos, 0);
        assert_eq!(events[1].sample_pos, 44_100);
        assert_eq!(events[2].sample_pos, 88_200);
    }
}