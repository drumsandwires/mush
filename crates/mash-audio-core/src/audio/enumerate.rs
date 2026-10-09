// mush/crates/mash-audio-core/src/audio/enumerate.rs

use std::collections::HashMap;

use cpal::traits::{DeviceTrait, HostTrait};

use super::types::{
    AudioDeviceError, AudioDeviceInfo, AudioHostInfo, AudioSupportedRates, AudioSystemInfo,
};

/// ALSA reports 1–4_294_967_295 Hz for virtual/plugin devices that have no
/// real hardware rate constraint. These entries are noise — drop them.
const ALSA_WILDCARD_MIN: u32 = 1;
const ALSA_WILDCARD_MAX: u32 = u32::MAX;

fn is_wildcard_rate(rates: &AudioSupportedRates) -> bool {
    rates.min == ALSA_WILDCARD_MIN && rates.max == ALSA_WILDCARD_MAX
}

pub fn enumerate_hosts_and_devices() -> Result<AudioSystemInfo, AudioDeviceError> {
    let mut hosts = Vec::new();

    for host_id in cpal::available_hosts() {
        let host = cpal::host_from_id(host_id)?;

        let default_in_id = host.default_input_device().and_then(|d| d.id().ok());
        let default_out_id = host.default_output_device().and_then(|d| d.id().ok());

        // Keyed by description — first entry wins, except a default-flagged
        // entry always displaces a non-default one with the same description.
        let mut seen: HashMap<String, AudioDeviceInfo> = HashMap::new();

        for device in host.devices()? {
            let description = device
                .description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| "<unknown>".to_string());

            let device_id = device.id().ok();

            let is_default_input = device_id
                .as_ref()
                .zip(default_in_id.as_ref())
                .map(|(a, b)| a == b)
                .unwrap_or(false);

            let is_default_output = device_id
                .as_ref()
                .zip(default_out_id.as_ref())
                .map(|(a, b)| a == b)
                .unwrap_or(false);

            let input_channels = device.default_input_config().ok().map(|c| c.channels());
            let output_channels = device.default_output_config().ok().map(|c| c.channels());

            let supported_rates = device
                .supported_output_configs()
                .ok()
                .and_then(|mut it| it.next())
                .or_else(|| {
                    device
                        .supported_input_configs()
                        .ok()
                        .and_then(|mut it| it.next())
                })
                .map(|cfg| AudioSupportedRates {
                    min: cfg.min_sample_rate(),
                    max: cfg.max_sample_rate(),
                });

            // Drop virtual/plugin devices with wildcard rate ranges.
            if supported_rates
                .as_ref()
                .map(is_wildcard_rate)
                .unwrap_or(false)
            {
                continue;
            }

            let info = AudioDeviceInfo {
                description: description.clone(),
                is_default_input,
                is_default_output,
                input_channels,
                output_channels,
                supported_rates,
            };

            // Keep this entry if:
            //   a) we haven't seen this description before, or
            //   b) this entry is a default and the existing one isn't.
            let keep = match seen.get(&description) {
                None => true,
                Some(existing) => {
                    (is_default_input || is_default_output)
                        && !existing.is_default_input
                        && !existing.is_default_output
                }
            };

            if keep {
                seen.insert(description, info);
            }
        }

        // Restore a stable order: defaults first, then alphabetical.
        let mut devices: Vec<AudioDeviceInfo> = seen.into_values().collect();
        devices.sort_by(|a, b| {
            let a_default = a.is_default_input || a.is_default_output;
            let b_default = b.is_default_input || b.is_default_output;
            b_default
                .cmp(&a_default)
                .then(a.description.cmp(&b.description))
        });

        hosts.push(AudioHostInfo {
            name: format!("{:?}", host_id),
            devices,
        });
    }

    Ok(AudioSystemInfo { hosts })
}
