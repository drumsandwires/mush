use mush_core::transport::{MidiEvent, TransportCommand};
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

use crate::MidiPlugin;

pub struct Play;

impl SimplePluginCommand for Play {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi play" }
    fn description(&self) -> &str {
        "Start MIDI playback; optionally pipe a list of events to queue"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::String)
            .input_output_type(
                Type::List(Box::new(Type::Record(vec![].into()))),
                Type::String,
            )
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let mut client = plugin.client.lock().expect("client lock poisoned");

        // ── Pipeline events ───────────────────────────────────────────────
        if let Value::List { vals, .. } = input {
            for val in vals {
                let event = midi_event_from_value(val)?;
                plugin.rt
                    .block_on(client.send_command(&TransportCommand::QueueEvent(event)))
                    .map_err(|e| LabeledError::new(e.to_string()))?;
            }
        }

        // ── Play ──────────────────────────────────────────────────────────
        plugin.rt
            .block_on(client.send_command(&TransportCommand::Play))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let update = plugin.rt.block_on(client.recv_update());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "ok".to_string());

        Ok(Value::string(s, Span::unknown()))
    }
}

/// Extract a [`MidiEvent`] from a Nu record value.
///
/// Expected columns: `beat`, `channel`, `note`, `vel`, `duration`.
/// All columns are required; types must be numeric.
fn midi_event_from_value(val: &Value) -> Result<MidiEvent, LabeledError> {
    let span = val.span();

    let get_f64 = |key: &str| -> Result<f64, LabeledError> {
        val.get_data_by_key(key)
            .ok_or_else(|| {
                LabeledError::new(format!("missing column '{key}'"))
                    .with_label(format!("record must have a '{key}' column"), span)
            })?
            .as_float()
            .map_err(|e| {
                LabeledError::new(format!("column '{key}': {e}"))
                    .with_label("expected a number", span)
            })
    };

    let get_u8 = |key: &str| -> Result<u8, LabeledError> {
        let v = val
            .get_data_by_key(key)
            .ok_or_else(|| {
                LabeledError::new(format!("missing column '{key}'"))
                    .with_label(format!("record must have a '{key}' column"), span)
            })?
            .as_int()
            .map_err(|e| {
                LabeledError::new(format!("column '{key}': {e}"))
                    .with_label("expected an integer", span)
            })?;
        u8::try_from(v).map_err(|_| {
            LabeledError::new(format!("column '{key}' value {v} out of range 0–127"))
                .with_label("must be 0–127", span)
        })
    };

    Ok(MidiEvent {
        beat:     get_f64("beat")?,
        channel:  get_u8("channel")?,
        note:     get_u8("note")?,
        vel:      get_u8("vel")?,
        duration: get_f64("duration")?,
    })
}