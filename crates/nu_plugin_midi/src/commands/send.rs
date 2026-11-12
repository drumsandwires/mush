use mush_core::transport::TransportCommand;
use mush_midi_codec::parse;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, SyntaxShape, Type, Value};

use crate::MidiPlugin;

pub struct Send;

impl SimplePluginCommand for Send {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi send" }
    fn description(&self) -> &str {
        "Send a raw MIDI message — symbolic (NoteOn C4 mf) or numeric (0x90 0x3C 0x64)"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "message",
                SyntaxShape::String,
                "MIDI message in symbolic or numeric form",
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
        let message: String = call.req(0)?;

        let bytes = parse(&message)
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let mut client = plugin.client.lock().expect("client lock poisoned");
        plugin.rt.block_on(client.send_command(&TransportCommand::SendMessage(bytes)))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let update = plugin.rt.block_on(client.recv_update());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "ok".to_string());

        Ok(Value::string(s, Span::unknown()))
    }
}