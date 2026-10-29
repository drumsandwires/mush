// mush/crates/mush-core/src/sink.rs
//
// EngineSink — the push-based counterpart to the pull-based Shell trait.
//
// Shell trait  (existing, lib.rs) — user input → engine
//                                   "what did the user type?"
//
// EngineSink trait (this file)   — engine → display
//                                  "what should be shown to the user?"
//
// These are deliberately separate traits with separate concerns.

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
