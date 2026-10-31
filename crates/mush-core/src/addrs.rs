/// ZMQ socket addresses for the EngineMonitor transport.
///
/// Use `INPROC` variants when shells and engine run in the same
/// process. Use `TCP` variants when running in separate processes.
/// The shell and transport code is identical in both cases —
/// only the address passed to `ZmqTransport::bind` and
/// `ZmqClient::connect` changes.
/// ROUTER socket — receives TransportCommand from shell DEALERs.
pub const CMD_TCP: &str = "tcp://localhost:5252";

/// PUB socket — sends EngineUpdate to all shell SUBscribers.
pub const PUB_TCP: &str = "tcp://localhost:5253";

/// Intra-process ROUTER — same process, no serialisation copy.
pub const CMD_INPROC: &str = "inproc://mush-cmd";

/// Intra-process PUB — same process, no serialisation copy.
pub const PUB_INPROC: &str = "inproc://mush-pub";
