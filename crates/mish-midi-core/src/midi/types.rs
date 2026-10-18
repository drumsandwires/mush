use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiPortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub struct MidiPortInfo {
    pub index: usize,
    pub name: String,
    pub direction: MidiPortDirection,
}

#[derive(Debug, Error)]
pub enum MidiError {
    #[error("failed to initialise MIDI input: {0}")]
    InputInit(midir::InitError),

    #[error("failed to initialise MIDI output: {0}")]
    OutputInit(midir::InitError),

    #[error("failed to read port name: {0}")]
    PortName(#[from] midir::PortInfoError),
}
