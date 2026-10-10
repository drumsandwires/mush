use super::types::{MidiError, MidiPortDirection, MidiPortInfo};
use midir::{Ignore, MidiInput, MidiOutput};

/// Returns all available MIDI input and output devices as structured data.
pub fn enumerate_devices() -> Result<Vec<MidiPortInfo>, MidiError> {
    let mut midi_in = MidiInput::new("mish-midi-core").map_err(MidiError::InputInit)?;
    midi_in.ignore(Ignore::None);

    let midi_out = MidiOutput::new("mish").map_err(MidiError::OutputInit)?;

    let inputs = midi_in.ports().into_iter().enumerate().map(|(index, p)| {
        Ok(MidiPortInfo {
            index,
            name: midi_in.port_name(&p)?,
            direction: MidiPortDirection::Input,
        })
    });

    let outputs = midi_out.ports().into_iter().enumerate().map(|(index, p)| {
        Ok(MidiPortInfo {
            index,
            name: midi_out.port_name(&p)?,
            direction: MidiPortDirection::Output,
        })
    });

    inputs.chain(outputs).collect()
}
