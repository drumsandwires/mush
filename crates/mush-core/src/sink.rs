// EngineSink — the push-based counterpart to the pull-based Shell trait.
//
// Shell trait  (existing, lib.rs) — user input → engine
//                                   "what did the user type?"
//
// EngineSink trait (this file)   — engine → display
//                                  "what should be shown to the user?"

use crate::update::EngineUpdate;

/// Receives EngineUpdates from the EngineMonitor and presents
/// them to the user.
///
/// # Implementors
/// - `ReedlineSink` — prints formatted output to the reedline REPL
/// - `NushellSink`  — forwards updates to a Nushell plugin subscription
///
/// # Thread safety
/// `EngineSink` requires `Send + Sync + 'static` because the
/// EngineMonitor holds a `Vec<Box<dyn EngineSink>>` and emits
/// from an async task that may run on any thread.
pub trait EngineSink: Send + Sync + 'static {
    /// Present an EngineUpdate to the user.
    /// Must not block — the EngineMonitor calls this from its
    /// async select loop.
    fn emit(&self, update: EngineUpdate);

    /// A human-readable name for this sink, used in logging.
    fn name(&self) -> &str;
}

/// Delivers [`EngineUpdate`]s to a reedline REPL without blocking.
///
/// Holds the sending end of a [`reedline::ExternalPrinter`] channel.
/// `main.rs` constructs the `ExternalPrinter`, passes the sender here
/// via [`ReedlineSink::new`], and registers the printer with
/// `Reedline::with_external_printer()`.
///
/// # Non-blocking guarantee
/// Uses `try_send` — if the channel is full the message is dropped
/// rather than blocking the Conductor async loop.
///
/// # Feature flag
/// Only compiled when the `reedline-sink` feature is enabled.
/// Shell crates opt in via:
/// ```toml
/// mush-core = { workspace = true, features = ["reedline-sink"] }
/// ```
#[cfg(feature = "reedline-sink")]
pub struct ReedlineSink {
    sender: std::sync::mpsc::SyncSender<String>,
    name: &'static str,
}

#[cfg(feature = "reedline-sink")]
impl ReedlineSink {
    pub fn new(sender: std::sync::mpsc::SyncSender<String>, name: &'static str) -> Self {
        Self { sender, name }
    }
}

#[cfg(feature = "reedline-sink")]
impl EngineSink for ReedlineSink {
    fn emit(&self, update: EngineUpdate) {
        if let Some(line) = crate::format::format_update(&update) {
            if let Err(e) = self.sender.try_send(line) {
                eprintln!("{} sink: dropped update — {e}", self.name);
            }
        }
    }

    fn name(&self) -> &str {
        self.name
    }
}
