pub mod addrs;
pub mod audio;
pub mod clock;
pub mod conductor;
pub mod format;
pub mod midi;
pub mod midi_output;
pub mod rt_priority;
pub mod shell;
pub mod sink;
pub mod subscription;
pub mod timeline;
pub mod transport;
pub mod update;
pub mod version;
pub mod zmq_client;
pub mod zmq_transport;

/// The response returned after evaluating one line of user input.
#[derive(Debug, PartialEq)]
pub enum Response {
    Output(String),
    Empty,
    Exit(String),
}
