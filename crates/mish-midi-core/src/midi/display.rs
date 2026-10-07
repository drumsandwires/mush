use super::types::{MidiPortDirection, MidiPortInfo};

/// Formats a MIDI port list for console display.
/// Returns `None` if the slice is empty.
pub fn list_enumerated_ports(ports: &[MidiPortInfo]) -> Option<String> {
    if ports.is_empty() {
        return None;
    }

    Some(
        ports
            .iter()
            .map(|p| {
                let dir = match p.direction {
                    MidiPortDirection::Input => "IN ",
                    MidiPortDirection::Output => "OUT",
                };
                format!("{dir} {}: {}", p.index, p.name)
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}
