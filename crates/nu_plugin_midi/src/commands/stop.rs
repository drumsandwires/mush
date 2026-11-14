use mush_core::transport::TransportCommand;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

use crate::MidiPlugin;

pub struct Stop;

impl SimplePluginCommand for Stop {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi stop" }
    fn description(&self) -> &str { "Stop MIDI playback or recording" }

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
        let mut handle = plugin.handle.lock().expect("handle lock poisoned");
        plugin.rt.block_on(client.send_command(&TransportCommand::Stop))
            .map_err(|e| LabeledError::new(e))?;

        let update = plugin.rt.block_on(handle.recv());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "ok".to_string());

        Ok(Value::string(s, Span::unknown()))
    }
}
