use mush_core::transport::TransportCommand;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, SyntaxShape, Type, Value};

use crate::MidiPlugin;

pub struct Port;

impl SimplePluginCommand for Port {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi port" }
    fn description(&self) -> &str { "Get or set the active MIDI output port" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional(
                "name",
                SyntaxShape::String,
                "Port name to select; omit to query the current port",
            )
            .input_output_type(Type::Nothing, Type::String)
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let name: Option<String> = call.opt(0)?;
        let mut client = plugin.client.lock().expect("client lock poisoned");
        let mut handle = plugin.handle.lock().expect("handle lock poisoned");

        plugin.rt.block_on(client.send_command(&TransportCommand::MidiPort(name)))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let update = plugin.rt.block_on(handle.recv());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "no response".to_string());

        Ok(Value::string(s, Span::unknown()))
    }
}