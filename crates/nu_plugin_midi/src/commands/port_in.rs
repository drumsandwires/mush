use mush_core::transport::TransportCommand;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, SyntaxShape, Type, Value};

use crate::MidiPlugin;

pub struct PortIn;

impl SimplePluginCommand for PortIn {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi port in" }
    fn description(&self) -> &str { "Set the active MIDI input port by name, index, or substring" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional(
                "selector",
                SyntaxShape::String,
                "Port name, integer index, or substring; omit to clear",
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
        let _span = tracing::debug_span!("command", cmd = "port in").entered();
        tracing::debug!("port in: run");

        let selector: Option<String> = call.opt(0)?;

        let mut client = plugin.client.lock().expect("client lock poisoned");
        let mut handle = plugin.handle.lock().expect("handle lock poisoned");

        plugin
            .rt
            .block_on(client.send_command(&TransportCommand::MidiInputPort(selector)))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let update = plugin.rt.block_on(handle.recv());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "no response".to_string());

        tracing::debug!(result = %s, "port in: done");
        Ok(Value::string(s, Span::unknown()))
    }
}