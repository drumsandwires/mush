// subscription.rs
//
// In-process update subscription for TheConductor.
//
// TheConductor holds a Vec<Subscription> and calls send() on each
// when an EngineUpdate is emitted. The corresponding
// SubscriptionHandle is held by the shell or in-process component
// and calls recv().await to consume updates.
//
// Built on tokio::sync::mpsc — integrates naturally with the
// tokio::select! loop in TheConductor.
//
// When a SubscriptionHandle is dropped, the next send() returns
// Err(()) and TheConductor removes the dead Subscription via
// retain().

use tokio::sync::mpsc;

use crate::update::EngineUpdate;

/// Conductor-side half of a subscription pair.
///
/// Held by `TheConductor` in a `Vec<Subscription>`.
/// Call `send()` to deliver an update to the subscriber.
/// Call `is_closed()` to check whether the handle has been dropped.
pub struct Subscription {
    tx: mpsc::Sender<EngineUpdate>,
}

impl Subscription {
    /// Send an update to the subscriber.
    ///
    /// Returns `Ok(())` if the update was queued.
    /// Returns `Err(())` if the `SubscriptionHandle` has been
    /// dropped or the channel is full.
    ///
    /// Never blocks — uses `try_send` on a bounded channel.
    pub fn send(&self, update: EngineUpdate) -> Result<(), ()> {
        self.tx.try_send(update).map_err(|_| ())
    }

    /// Returns `true` if the `SubscriptionHandle` has been dropped.
    ///
    /// `TheConductor` uses this in `retain()` to clean up dead
    /// subscriptions before the next broadcast.
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed()
    }
}

/// Subscriber-side half of a subscription pair.
///
/// Held by the shell or in-process component.
/// Call `recv().await` to wait for the next update.
pub struct SubscriptionHandle {
    rx: mpsc::Receiver<EngineUpdate>,
}

impl SubscriptionHandle {
    /// Receive the next update.
    ///
    /// Suspends until an update arrives or `TheConductor` shuts
    /// down and all pending updates have been consumed.
    /// Returns `None` when the channel is closed and drained.
    pub async fn recv(&mut self) -> Option<EngineUpdate> {
        self.rx.recv().await
    }
}

/// Create a linked `Subscription` / `SubscriptionHandle` pair.
///
/// `capacity` is the number of updates that can be queued before
/// `send()` starts returning `Err`. 32 is sufficient for a REPL
/// that processes updates on every iteration.
///
/// # Example
/// ```rust
/// use mush_core::subscription::subscription;
///
/// let (sub, mut handle) = subscription(32);
/// // Give sub to TheConductor, handle to the shell.
/// ```
pub fn subscription(capacity: usize) -> (Subscription, SubscriptionHandle) {
    let (tx, rx) = mpsc::channel(capacity);
    (Subscription { tx }, SubscriptionHandle { rx })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::{AudioUpdate, EngineUpdate};

    // 6.1.3 — Subscription::send() delivers to handle
    #[tokio::test]
    async fn send_delivers_to_handle() {
        let (sub, mut handle) = subscription(32);
        let update = EngineUpdate::Audio(AudioUpdate::TransportStarted);

        sub.send(update.clone()).unwrap();
        let received = handle.recv().await.unwrap();

        assert_eq!(received, update);
    }

    // 6.1.4 — SubscriptionHandle::recv() returns None when closed
    #[tokio::test]
    async fn recv_returns_none_when_closed() {
        let (sub, mut handle) = subscription(32);
        drop(sub);
        assert!(handle.recv().await.is_none());
    }

    // 6.1.5 — dropped handle detected via is_closed()
    #[test]
    fn is_closed_after_handle_dropped() {
        let (sub, handle) = subscription(32);
        assert!(!sub.is_closed());
        drop(handle);
        assert!(sub.is_closed());
    }

    // 6.1.5 — dropped handle detected via send() returning Err
    #[test]
    fn send_returns_err_after_handle_dropped() {
        let (sub, handle) = subscription(32);
        drop(handle);
        let result = sub.send(EngineUpdate::Audio(AudioUpdate::TransportStopped));
        assert!(result.is_err());
    }

    // 6.2.13 — retain() removes closed subscriptions
    #[test]
    fn retain_removes_closed_subscriptions() {
        let (sub1, _handle1) = subscription(32);
        let (sub2, handle2) = subscription(32);
        let (sub3, _handle3) = subscription(32);

        // Drop handle2 — sub2 should be removed by retain()
        drop(handle2);

        let mut subs = vec![sub1, sub2, sub3];
        subs.retain(|s| !s.is_closed());

        assert_eq!(subs.len(), 2);
    }
}
