use futures::{SinkExt, StreamExt};
use tmq::{Context, dealer, subscribe};

use crate::transport::TransportCommand;
use crate::update::EngineUpdate;

/// Client-side ZMQ transport.
///
/// One instance per shell. Connect to the `EngineMonitor` with
/// `ZmqClient::connect()` before sending commands or receiving
/// updates.
pub struct ZmqClient {
    dealer: tmq::dealer::Dealer,
    subscribe: tmq::subscribe::Subscribe,
}

impl ZmqClient {
    /// Connect both sockets to the `EngineMonitor`.
    ///
    /// # Arguments
    /// - `cmd_addr` — address of the ROUTER socket to connect to
    /// - `pub_addr` — address of the PUB socket to subscribe to
    ///
    /// # Examples
    /// ```rust,no_run
    /// // Same process
    /// ZmqClient::connect(addrs::CMD_INPROC, addrs::PUB_INPROC)?;
    ///
    /// // Separate process
    /// ZmqClient::connect(addrs::CMD_TCP, addrs::PUB_TCP)?;
    /// ```
    ///
    /// # Errors
    /// Returns `Err` if either socket fails to connect.
    pub fn connect(cmd_addr: &str, pub_addr: &str) -> Result<Self, tmq::TmqError> {
        let ctx = Context::new();

        let dealer = dealer(&ctx).connect(cmd_addr)?;

        let subscribe = subscribe(&ctx).connect(pub_addr)?.subscribe(b"")?;

        Ok(Self { dealer, subscribe })
    }

    /// Send a `TransportCommand` to the `EngineMonitor`.
    pub async fn send_command(&mut self, cmd: &TransportCommand) -> Result<(), String> {
        let payload = rmp_serde::to_vec(cmd).map_err(|e| e.to_string())?;

        // DEALER frame: [empty delimiter, payload]
        self.dealer
            .send(vec![vec![], payload])
            .await
            .map_err(|e| e.to_string())
    }

    /// Receive the next `EngineUpdate` from the `EngineMonitor`.
    ///
    /// Suspends until a message arrives — does not block the thread.
    /// Returns `None` if the socket is closed or the message cannot
    /// be deserialised.
    pub async fn recv_update(&mut self) -> Option<EngineUpdate> {
        let msg = self.subscribe.next().await?;
        let msg = msg.ok()?;

        // SUB frame: [topic, payload]
        let frames: Vec<_> = msg.into_iter().collect();
        let payload = frames.get(1)?;

        rmp_serde::from_slice(payload).ok()
    }
}
