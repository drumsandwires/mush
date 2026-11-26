// EngineSink — the push-based counterpart to the pull-based Shell trait.
//
// Shell trait  (existing, lib.rs) — user input → engine
//                                   "what did the user type?"
//
// EngineSink trait (this file)   — engine → display
//                                  "what should be shown to the user?"
use crate::update::{EngineUpdate, MidiUpdate};

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
/// Only compiled when the `reedline-shell` feature is enabled.
/// Shell crates opt in via:
/// ```toml
/// mush-core = { workspace = true, features = ["reedline-shell"] }
/// ```
#[cfg(feature = "reedline-shell")]
pub struct ReedlineSink {
    sender: std::sync::mpsc::SyncSender<String>,
    name: &'static str,
}

#[cfg(feature = "reedline-shell")]
impl ReedlineSink {
    pub fn new(sender: std::sync::mpsc::SyncSender<String>, name: &'static str) -> Self {
        Self { sender, name }
    }
}

#[cfg(feature = "reedline-shell")]
impl EngineSink for ReedlineSink {
    fn emit(&self, update: EngineUpdate) {
        // EventDispatched is filtered here — never forwarded to the reedline
        // printer. Two reasons:
        //
        // 1. Volume — at performance tempo, EventDispatched arrives at
        //    10–100 Hz. The ExternalPrinter channel (capacity 32) fills in
        //    under 2 seconds and begins dropping with warnings.
        //
        // 2. Context — a scrolling REPL line has no useful presentation for
        //    a raw MIDI event. The live view TUI (mush_core::live_view) owns
        //    event display via the ZMQ SUB socket and the alternate screen.
        //
        // TransportStatus is also filtered — it is consumed by the live view
        // TUI header and has no useful one-line REPL representation beyond
        // what the existing ActivePort / Error updates already provide.
        //
        // See DESIGN.md §Monitor Architecture for full rationale.
        match &update {
            EngineUpdate::Midi(MidiUpdate::EventDispatched { .. }) => return,
            EngineUpdate::Transport(_) => return,
            _ => {}
        }

        if let Some(line) = crate::format::format_update(&update) {
            if let Err(e) = self.sender.try_send(line) {
                tracing::warn!(sink = self.name, error = %e, "dropped update");
            }
        }
    }

    fn name(&self) -> &str {
        self.name
    }
}
