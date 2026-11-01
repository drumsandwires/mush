pub mod addrs;
pub mod conductor;
pub mod format;
pub mod midi_output;
pub mod rt_priority;
pub mod sink;
pub mod subscription;
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