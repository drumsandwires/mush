use mush_core::midi::types::MidiPortDirection;
use mush_core::transport::TransportCommand;
use mush_core::update::{EngineUpdate, MidiUpdate};
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Record, Signature, Span, Type, Value};

use crate::MidiPlugin;

pub struct Ports;

impl SimplePluginCommand for Ports {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str {
        "midi ports"
    }
    fn description(&self) -> &str {
        "List available MIDI ports (input, output, and thru)"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name()).input_output_type(Type::Nothing, Type::Table(vec![].into()))
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let _span = tracing::debug_span!("command", cmd = "ports").entered();
        tracing::debug!("ports: run");

        let mut client = plugin.client.lock().expect("client lock poisoned");
        let mut handle = plugin.handle.lock().expect("handle lock poisoned");

        plugin
            .rt
            .block_on(client.send_command(&TransportCommand::MidiDevices))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let update = plugin.rt.block_on(handle.recv());
        let ports = match update {
            Some(EngineUpdate::Midi(MidiUpdate::PortList(ports))) => ports,
            _ => {
                return Err(LabeledError::new(
                    "unexpected or missing response from conductor",
                ));
            }
        };

        let span = Span::unknown();
        let rows = ports
            .into_iter()
            .map(|port| {
                let direction = match port.direction {
                    MidiPortDirection::Input => "IN",
                    MidiPortDirection::Output => "OUT",
                    MidiPortDirection::Thru => "THRU",
                };
                let mut record = Record::new();
                record.push("direction", Value::string(direction, span));
                record.push("index", Value::int(port.index as i64, span));
                record.push("name", Value::string(port.name, span));
                Value::record(record, span)
            })
            .collect::<Vec<_>>();

        tracing::debug!("ports: done");
        Ok(Value::list(rows, span))
    }
}
