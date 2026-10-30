// Server-side ZMQ transport for the EngineMonitor.
//
// Binds two sockets:
//   ROUTER tcp://*:5252  — receives TransportCommand from shell DEALERs
//   PUB    tcp://*:5253  — publishes EngineUpdate to shell SUBscribers
//
// Serialisation: msgpack via rmp-serde.

use futures::SinkExt;
use tmq::publish::Publish;
use tmq::router::Router;
use tmq::{Context, publish, router};

use crate::transport::TransportCommand;
use crate::update::EngineUpdate;

/// Server-side ZMQ transport.
///
/// Owned by the `EngineMonitor`. Not `Clone` — only one instance
/// should exist per process.
pub struct ZmqTransport {
    router: Router,
    pub_socket: Publish,
}

impl ZmqTransport {
    /// Bind both sockets and return a ready-to-use transport.
    ///
    /// # Errors
    /// Returns `Err` if either socket fails to bind — typically
    /// because the port is already in use.
    pub fn bind() -> Result<Self, tmq::TmqError> {
        let ctx = Context::new();

        let router = router(&ctx).bind("tcp://*:5252")?;

        let pub_socket = publish(&ctx).bind("tcp://*:5253")?;

        Ok(Self { router, pub_socket })
    }

    /// Receive the next `TransportCommand` from any connected shell.
    ///
    /// Suspends until a message arrives — does not block the thread.
    /// Returns `None` if the socket is closed or the message cannot
    /// be deserialised.
    pub async fn recv_command(&mut self) -> Option<TransportCommand> {
        use futures::StreamExt;

        let msg = self.router.next().await?;
        let msg = msg.ok()?;

        // ROUTER receives: [identity, empty delimiter, payload]
        // Collect into Vec so we can index by position
        let frames: Vec<_> = msg.into_iter().collect();
        let payload = frames.get(2)?;

        rmp_serde::from_slice(payload).ok()
    }

    /// Publish an `EngineUpdate` to all connected shell subscribers.
    ///
    /// PUB/SUB in ZMQ requires a topic prefix. We use an empty topic
    /// so all subscribers receive all messages — shells subscribe with
    /// an empty filter.
    pub async fn send_update(&mut self, update: &EngineUpdate) -> Result<(), String> {
        let payload = rmp_serde::to_vec(update).map_err(|e| e.to_string())?;

        // PUB frame: [topic, payload]
        // Empty topic — all subscribers receive.
        self.pub_socket
            .send(vec![vec![], payload])
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::{AudioUpdate, EngineUpdate};

    // ── Msgpack roundtrip tests ───────────────────────────────────────────

    #[test]
    fn roundtrip_transport_command_play() {
        let cmd = TransportCommand::Play;
        let bytes = rmp_serde::to_vec(&cmd).unwrap();
        let decoded: TransportCommand = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(cmd, decoded);
    }

    #[test]
    fn roundtrip_transport_command_stop() {
        let cmd = TransportCommand::Stop;
        let bytes = rmp_serde::to_vec(&cmd).unwrap();
        let decoded: TransportCommand = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(cmd, decoded);
    }

    #[test]
    fn roundtrip_transport_command_set_bpm() {
        let cmd = TransportCommand::SetBpm(140.0);
        let bytes = rmp_serde::to_vec(&cmd).unwrap();
        let decoded: TransportCommand = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(cmd, decoded);
    }

    #[test]
    fn roundtrip_transport_command_load_midi() {
        let cmd = TransportCommand::LoadMidi("/tmp/test.mid".into());
        let bytes = rmp_serde::to_vec(&cmd).unwrap();
        let decoded: TransportCommand = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(cmd, decoded);
    }

    #[test]
    fn roundtrip_engine_update_audio() {
        let update = EngineUpdate::Audio(AudioUpdate::PlayheadMoved(44_100));
        let bytes = rmp_serde::to_vec(&update).unwrap();
        let decoded: EngineUpdate = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(update, decoded);
    }

    #[test]
    fn roundtrip_engine_update_error() {
        let update = EngineUpdate::Audio(AudioUpdate::Error("test error".into()));
        let bytes = rmp_serde::to_vec(&update).unwrap();
        let decoded: EngineUpdate = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(update, decoded);
    }
}
