use super::types::AudioSystemInfo;

pub fn list_hosts_and_devices(info: &AudioSystemInfo) -> String {
    let mut out = String::new();

    for host in &info.hosts {
        out.push_str(&format!("Host: {}\n", host.name));

        if host.devices.is_empty() {
            out.push_str("  (no devices found)\n");
            continue;
        }

        for device in &host.devices {
            let mut tags = Vec::new();
            if device.is_default_input {
                tags.push("default in");
            }
            if device.is_default_output {
                tags.push("default out");
            }
            let tag_str = if tags.is_empty() {
                String::new()
            } else {
                format!("  [{}]", tags.join(", "))
            };

            out.push_str(&format!("  • {}{}\n", device.description, tag_str));

            match (device.input_channels, device.output_channels) {
                (Some(i), Some(o)) if i == o => {
                    out.push_str(&format!("\tchannels:        {}\n", i));
                }
                (Some(i), Some(o)) => {
                    out.push_str(&format!("\tinput channels:  {}\n", i));
                    out.push_str(&format!("\toutput channels: {}\n", o));
                }
                (Some(i), None) => {
                    out.push_str(&format!("\tinput channels:  {}\n", i));
                }
                (None, Some(o)) => {
                    out.push_str(&format!("\toutput channels: {}\n", o));
                }
                (None, None) => {}
            }

            if let Some(rates) = &device.supported_rates {
                if rates.min == rates.max {
                    out.push_str(&format!("\tsample rate:\t{} Hz\n", rates.min));
                } else {
                    out.push_str(&format!(
                        "\tsample rates:\t{}–{} Hz\n",
                        rates.min, rates.max
                    ));
                }
            }
        }
    }

    out
}
