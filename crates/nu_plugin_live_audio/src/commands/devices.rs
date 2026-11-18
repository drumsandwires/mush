use crate::AudioPlugin;
use mush_core::transport::TransportCommand;
use mush_core::update::{AudioUpdate, EngineUpdate};
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Record, Signature, Span, Type, Value};

pub struct Devices;

impl SimplePluginCommand for Devices {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str { "audio devices" }
    fn description(&self) -> &str { "List available audio hosts and devices" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::Table(vec![].into()))
    }

    fn run(
        &self,
        plugin: &AudioPlugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let _span = tracing::debug_span!("command", cmd = "devices").entered();
        tracing::debug!("devices: run");
        let mut client = plugin.client.lock().expect("client lock poisoned");
        let mut handle = plugin.handle.lock().expect("handle lock poisoned");

        plugin.rt
            .block_on(client.send_command(&TransportCommand::AudioDevices))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let info = match plugin.rt.block_on(handle.recv()) {
            Some(EngineUpdate::Audio(AudioUpdate::DeviceList(info))) => info,
            _ => return Err(LabeledError::new("unexpected or missing response from conductor")),
        };

        let span = Span::unknown();

        let rows = info.hosts.into_iter().map(|host| {
            let devices = host.devices.into_iter().map(|device| {
                let mut rec = Record::new();
                rec.push("name",             Value::string(device.description, span));
                rec.push("default_input",    Value::bool(device.is_default_input, span));
                rec.push("default_output",   Value::bool(device.is_default_output, span));
                rec.push("input_channels",   match device.input_channels {
                    Some(c) => Value::int(c as i64, span),
                    None    => Value::nothing(span),
                });
                rec.push("output_channels",  match device.output_channels {
                    Some(c) => Value::int(c as i64, span),
                    None    => Value::nothing(span),
                });
                rec.push("sample_rate_min",  match device.supported_rates.as_ref() {
                    Some(r) => Value::int(r.min as i64, span),
                    None    => Value::nothing(span),
                });
                rec.push("sample_rate_max",  match device.supported_rates.as_ref() {
                    Some(r) => Value::int(r.max as i64, span),
                    None    => Value::nothing(span),
                });
                Value::record(rec, span)
            }).collect::<Vec<_>>();

            let mut rec = Record::new();
            rec.push("host",    Value::string(host.name, span));
            rec.push("devices", Value::list(devices, span));
            Value::record(rec, span)
        }).collect::<Vec<_>>();

        tracing::debug!("devices: done");
        Ok(Value::list(rows, span))
    }
}