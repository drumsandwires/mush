use mush_core::transport::TransportCommand;
use mush_core::update::{EngineUpdate, MidiUpdate};
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Record, Signature, Span, Type, Value};
use crate::MidiPlugin;

pub struct Ports;

impl SimplePluginCommand for Ports {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi ports" }
    fn description(&self) -> &str { "List available MIDI output ports" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::Table(vec![].into()))
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let mut client = plugin.client.lock().expect("client lock poisoned");
        let mut handle = plugin.handle.lock().expect("handle lock poisoned");

        plugin.rt.block_on(client.send_command(&TransportCommand::MidiDevices))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let update = plugin.rt.block_on(handle.recv());

        let port_list = match update {
            Some(EngineUpdate::Midi(MidiUpdate::PortList(s))) => s,
            _ => return Err(LabeledError::new("unexpected or missing response from conductor")),
        };

        let span = Span::unknown();
        let rows = port_list
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| {
                // Format: "IN  0: Name" or "OUT 0: Name"
                let (direction, rest) = if line.starts_with("IN") {
                    ("IN", line.trim_start_matches("IN").trim())
                } else if line.starts_with("OUT") {
                    ("OUT", line.trim_start_matches("OUT").trim())
                } else {
                    ("?", line.trim())
                };

                // rest is "0: Port Name"
                let (index, name) = rest.split_once(':')
                    .map(|(i, n)| (i.trim(), n.trim()))
                    .unwrap_or(("?", rest));

                let mut record = Record::new();
                record.push("direction", Value::string(direction, span));
                record.push("index", Value::int(index.parse().unwrap_or(-1), span));
                record.push("name", Value::string(name, span));
                Value::record(record, span)
            })
            .collect::<Vec<_>>();

        Ok(Value::list(rows, span))
    }
}