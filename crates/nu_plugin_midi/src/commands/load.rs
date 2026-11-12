use std::path::PathBuf;
use mush_core::transport::TransportCommand;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, SyntaxShape, Type, Value};

use crate::MidiPlugin;

pub struct Load;

impl SimplePluginCommand for Load {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi load" }
    fn description(&self) -> &str { "Load a MIDI file (.mid or .mudl)" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("path", SyntaxShape::Filepath, "Path to the MIDI file")
            .input_output_type(Type::Nothing, Type::String)
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let path: String = call.req(0)?;
        let path = PathBuf::from(&path);

        let cmd = match path.extension().and_then(|e| e.to_str()) {
            Some("mudl") => TransportCommand::LoadMudl(path),
            _            => TransportCommand::LoadMidi(path),
        };

        let mut client = plugin.client.lock().expect("client lock poisoned");
        plugin.rt.block_on(client.send_command(&cmd))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let update = plugin.rt.block_on(client.recv_update());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "no response".to_string());

        Ok(Value::string(s, Span::unknown()))
    }
}