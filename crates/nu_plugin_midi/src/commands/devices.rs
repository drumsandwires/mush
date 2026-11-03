use mush_core::transport::TransportCommand;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

use crate::MidiPlugin;

pub struct Devices;

impl SimplePluginCommand for Devices {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi devices" }
    fn description(&self) -> &str { "List available MIDI devices" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::String)
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let mut client = plugin.client.lock().expect("client lock poisoned");
        plugin.rt.block_on(client.send_command(&TransportCommand::MidiDevices))
            .map_err(|e| LabeledError::new(e))?;

        let update = plugin.rt.block_on(client.recv_update());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "no response".to_string());

        Ok(Value::string(s, Span::unknown()))
    }
}
