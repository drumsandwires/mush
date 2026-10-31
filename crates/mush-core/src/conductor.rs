use std::sync::{Arc, RwLock};

use tokio::sync::mpsc;

use crate::addrs;
use crate::subscription::{Subscription, subscription};
use crate::transport::TransportCommand;
use crate::update::{AudioUpdate, EngineUpdate, MidiUpdate};
use crate::zmq_transport::ZmqTransport;

/// The central coordinator for the mush engine.
///
/// Owns the ZMQ transport, receives updates from the Clock and
/// MidiEngineRt via tokio mpsc channels, and fans out EngineUpdates
/// to all registered subscribers.
///
/// # Lifecycle
/// 1. Create with `Conductor::new()`
/// 2. Pass the returned senders to `Clock` and `MidiEngineRt`
/// 3. Call `Conductor::run().await` — runs until shutdown
///
/// # Example
/// ```rust,no_run
/// let (conductor, audio_tx, midi_tx) = Conductor::new(
///     addrs::CMD_INPROC,
///     addrs::PUB_INPROC,
/// ).unwrap();
/// conductor.run().await;
/// ```
pub struct Conductor {
    transport: ZmqTransport,
    audio_rx: mpsc::Receiver<AudioUpdate>,
    midi_rx: mpsc::Receiver<MidiUpdate>,
    subs: Vec<Subscription>,
}

/// Capacity of the audio and MIDI update channels.
/// 64 updates is sufficient for a 100ms reporting interval at
/// any realistic tempo — the conductor drains these on every
/// select! iteration.
const CHANNEL_CAPACITY: usize = 64;

impl Conductor {
    /// Create a new `Conductor` and return it alongside the
    /// senders that the `Clock` and `MidiEngineRt` write into.
    ///
    /// # Arguments
    /// - `cmd_addr` — address to bind the ROUTER socket
    /// - `pub_addr` — address to bind the PUB socket
    ///
    /// Use `addrs::CMD_INPROC` / `addrs::PUB_INPROC` for same-process
    /// deployment (MVP). Use `addrs::CMD_TCP` / `addrs::PUB_TCP` for
    /// standalone daemon deployment (post-MVP).
    ///
    /// # Errors
    /// Returns `Err` if either ZMQ socket fails to bind.
    pub fn new(
        cmd_addr: &str,
        pub_addr: &str,
    ) -> Result<(Self, mpsc::Sender<AudioUpdate>, mpsc::Sender<MidiUpdate>), tmq::TmqError> {
        let transport = ZmqTransport::bind(cmd_addr, pub_addr)?;

        let (audio_tx, audio_rx) = mpsc::channel(CHANNEL_CAPACITY);
        let (midi_tx, midi_rx) = mpsc::channel(CHANNEL_CAPACITY);

        let conductor = Self {
            transport,
            audio_rx,
            midi_rx,
            subs: Vec::new(),
        };

        Ok((conductor, audio_tx, midi_tx))
    }

    /// Register an in-process subscriber.
    ///
    /// Returns a `SubscriptionHandle` the caller uses to receive
    /// `EngineUpdate`s. Must be called before `run()`.
    pub fn subscribe(&mut self) -> crate::subscription::SubscriptionHandle {
        let (sub, handle) = subscription(32);
        self.subs.push(sub);
        handle
    }

    /// Run the conductor loop.
    ///
    /// Suspends on `tokio::select!` — wakes only when a command
    /// arrives or an update is ready to fan out. Returns when the
    /// ZMQ transport is closed or all senders are dropped.
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                cmd = self.transport.recv_command() => {
                    match cmd {
                        Some(cmd) => self.handle_command(cmd).await,
                        None      => break,
                    }
                }

                upd = self.audio_rx.recv() => {
                    match upd {
                        Some(upd) => self.emit_all(EngineUpdate::Audio(upd)).await,
                        None      => break,
                    }
                }

                upd = self.midi_rx.recv() => {
                    match upd {
                        Some(upd) => self.emit_all(EngineUpdate::Midi(upd)).await,
                        None      => break,
                    }
                }
            }
        }
    }

    /// Dispatch a `TransportCommand` to the appropriate handler.
    async fn handle_command(&mut self, cmd: TransportCommand) {
        match cmd {
            TransportCommand::Play => {
                self.emit_all(EngineUpdate::Audio(AudioUpdate::TransportStarted))
                    .await;
            }

            TransportCommand::Stop => {
                self.emit_all(EngineUpdate::Audio(AudioUpdate::TransportStopped))
                    .await;
            }

            TransportCommand::SetBpm(bpm) => {
                // TODO 6.2.6: write to TransportState atomics
                let _ = bpm;
            }

            TransportCommand::Seek(pos) => {
                // TODO 6.2.6: write to TransportState atomics
                let _ = pos;
            }

            TransportCommand::LoadMidi(path) => {
                // TODO 6.2.7: parse MIDI file, rebuild timeline
                let _ = path;
            }

            TransportCommand::LoadMudl(path) => {
                // Reserved — task 8
                let _ = path;
                self.emit_all(EngineUpdate::Audio(AudioUpdate::Error(
                    "LoadMudl not yet implemented".into(),
                )))
                .await;
            }

            TransportCommand::QueueEvent(event) => {
                // TODO 6.2.9: append to timeline or dispatch immediately
                let _ = event;
            }

            TransportCommand::AudioDevices => {
                // TODO 6.2.10: enumerate audio devices, emit DeviceList
            }

            TransportCommand::MidiDevices => {
                // TODO 6.2.11: enumerate MIDI ports, emit PortList
            }
        }
    }

    /// Fan out an `EngineUpdate` to all subscribers and the ZMQ PUB socket.
    ///
    /// Removes dead subscriptions (dropped handles) via `retain()`.
    async fn emit_all(&mut self, update: EngineUpdate) {
        // Fan out to in-process subscribers first.
        // retain() removes any whose handle has been dropped.
        self.subs.retain(|s| !s.is_closed());
        for sub in &self.subs {
            let _ = sub.send(update.clone());
        }

        // Publish to ZMQ PUB socket — reaches all ZMQ SUB subscribers.
        let _ = self.transport.send_update(&update).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addrs;
    use crate::update::{AudioUpdate, EngineUpdate, MidiUpdate};

    /// Helper — create a Conductor bound on inproc addresses.
    /// Returns the conductor and both status senders.
    fn make_conductor() -> (
        Conductor,
        mpsc::Sender<AudioUpdate>,
        mpsc::Sender<MidiUpdate>,
    ) {
        Conductor::new(addrs::CMD_INPROC, addrs::PUB_INPROC).unwrap()
    }

    // 6.3.1 — Play command emits TransportStarted
    #[tokio::test]
    async fn play_emits_transport_started() {
        let (mut conductor, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();

        conductor.handle_command(TransportCommand::Play).await;

        let update = handle.recv().await.unwrap();
        assert_eq!(update, EngineUpdate::Audio(AudioUpdate::TransportStarted));
    }

    // 6.3.2 — Stop command emits TransportStopped
    #[tokio::test]
    async fn stop_emits_transport_stopped() {
        let (mut conductor, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();

        conductor.handle_command(TransportCommand::Stop).await;

        let update = handle.recv().await.unwrap();
        assert_eq!(update, EngineUpdate::Audio(AudioUpdate::TransportStopped));
    }

    // 6.3.3 — SetBpm does not panic (stub — no state yet)
    #[tokio::test]
    async fn set_bpm_does_not_panic() {
        let (mut conductor, _audio_tx, _midi_tx) = make_conductor();
        conductor
            .handle_command(TransportCommand::SetBpm(120.0))
            .await;
    }

    // 6.3.5 — Update reaches SubscriptionHandle
    #[tokio::test]
    async fn update_reaches_subscription_handle() {
        let (mut conductor, audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();

        audio_tx
            .send(AudioUpdate::PlayheadMoved(44_100))
            .await
            .unwrap();

        // Drive the conductor for one iteration
        conductor
            .emit_all(EngineUpdate::Audio(AudioUpdate::PlayheadMoved(44_100)))
            .await;

        let update = handle.recv().await.unwrap();
        assert_eq!(
            update,
            EngineUpdate::Audio(AudioUpdate::PlayheadMoved(44_100))
        );
    }

    // 6.3.6 — Closed subscription is removed cleanly
    #[tokio::test]
    async fn closed_subscription_removed_cleanly() {
        let (mut conductor, _audio_tx, _midi_tx) = make_conductor();
        let handle = conductor.subscribe();

        // Drop the handle — subscription should be cleaned up
        drop(handle);

        // emit_all calls retain() — should not panic
        conductor
            .emit_all(EngineUpdate::Audio(AudioUpdate::TransportStopped))
            .await;

        assert_eq!(conductor.subs.len(), 0);
    }

    // 6.3.10 — Multiple subscriptions all receive update
    #[tokio::test]
    async fn multiple_subscriptions_all_receive_update() {
        let (mut conductor, _audio_tx, _midi_tx) = make_conductor();
        let mut handle_a = conductor.subscribe();
        let mut handle_b = conductor.subscribe();
        let mut handle_c = conductor.subscribe();

        conductor
            .emit_all(EngineUpdate::Audio(AudioUpdate::TransportStarted))
            .await;

        let expected = EngineUpdate::Audio(AudioUpdate::TransportStarted);
        assert_eq!(handle_a.recv().await.unwrap(), expected);
        assert_eq!(handle_b.recv().await.unwrap(), expected);
        assert_eq!(handle_c.recv().await.unwrap(), expected);
    }
}
