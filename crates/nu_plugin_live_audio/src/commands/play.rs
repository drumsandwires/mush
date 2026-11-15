use crate::AudioPlugin;
use mush_core::transport::TransportCommand;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

pub struct Play;

impl SimplePluginCommand for Play {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str { "audio play" }
    fn description(&self) -> &str { "Start audio playback" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::Nothing)
    }

    fn run(
        &self,
        plugin: &AudioPlugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let mut client = plugin.client.lock().expect("client lock poisoned");
        let mut handle = plugin.handle.lock().expect("handle lock poisoned");
        plugin.rt
            .block_on(client.send_command(&TransportCommand::Play))
            .map_err(|e| LabeledError::new(e.to_string()))?;
        let _ = plugin.rt.block_on(handle.recv());
        Ok(Value::nothing(Span::unknown()))
    }
}