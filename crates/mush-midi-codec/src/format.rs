use crate::Style;
use crate::error::FormatError;
use crate::note::note_to_str;
use crate::status::StatusKind;
use crate::velocity::velocity_to_str;

/// Format raw MIDI wire bytes into a human-readable string.
pub fn format(bytes: &[u8], style: Style) -> Result<String, FormatError> {
    if bytes.is_empty() {
        return Err(FormatError::EmptyInput);
    }

    let status_byte = bytes[0];

    // ── SysEx ────────────────────────────────────────────────────────────────
    if status_byte == 0xF0 {
        // Strip 0xF0, retain 0xF7, format payload as 0x-prefixed hex.
        let payload = &bytes[1..];
        if payload.is_empty() {
            return Err(FormatError::TruncatedMessage {
                status: 0xF0,
                expected: 2, // at minimum one payload byte + 0xF7
                got: 1,
            });
        }
        let hex: Vec<String> = payload.iter().map(|b| format!("0x{b:02X}")).collect();
        return Ok(format!("SysEx {}", hex.join(" ")));
    }

    // ── Channel voice ────────────────────────────────────────────────────────
    let (kind, channel_idx) =
        StatusKind::from_status_byte(status_byte).ok_or(FormatError::UnknownStatus(status_byte))?;

    let channel = channel_idx + 1; // convert to 1-indexed

    let expected = kind.data_len() + 1; // +1 for status byte
    if bytes.len() < expected {
        return Err(FormatError::TruncatedMessage {
            status: status_byte,
            expected,
            got: bytes.len(),
        });
    }

    let name = match style {
        Style::Long => kind.long_name(),
        Style::Short => kind.short_name(),
    };

    // Channel suffix — omit for channel 1.
    let ch_suffix = if channel == 1 {
        String::new()
    } else {
        format!("/{channel}")
    };

    let result = match kind {
        StatusKind::NoteOn | StatusKind::NoteOff => {
            let note = note_to_str(bytes[1]);
            let vel = velocity_to_str(bytes[2]);
            format!("{name}{ch_suffix} {note} {vel}")
        }
        StatusKind::PolyPressure => {
            let note = note_to_str(bytes[1]);
            let pressure = bytes[2];
            format!("{name}{ch_suffix} {note} {pressure}")
        }
        StatusKind::ControlChange => {
            format!("{name}{ch_suffix} {} {}", bytes[1], bytes[2])
        }
        StatusKind::ProgramChange | StatusKind::ChannelPressure => {
            format!("{name}{ch_suffix} {}", bytes[1])
        }
        StatusKind::PitchBend => {
            format!("{name}{ch_suffix} {} {}", bytes[1], bytes[2])
        }
        StatusKind::SysEx => unreachable!(),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── NoteOn ───────────────────────────────────────────────────────────────

    #[test]
    fn noteon_long_ch1() {
        assert_eq!(
            format(&[0x90, 60, 80], Style::Long).unwrap(),
            "NoteOn C4 mf"
        );
    }

    #[test]
    fn noteon_short_ch1() {
        assert_eq!(format(&[0x90, 60, 80], Style::Short).unwrap(), "Non C4 mf");
    }

    #[test]
    fn noteon_long_ch2() {
        assert_eq!(
            format(&[0x91, 60, 80], Style::Long).unwrap(),
            "NoteOn/2 C4 mf"
        );
    }

    #[test]
    fn noteon_long_ch16() {
        assert_eq!(
            format(&[0x9F, 60, 80], Style::Long).unwrap(),
            "NoteOn/16 C4 mf"
        );
    }

    // ── NoteOff ──────────────────────────────────────────────────────────────

    #[test]
    fn noteoff_long() {
        assert_eq!(
            format(&[0x80, 60, 48], Style::Long).unwrap(),
            "NoteOff C4 p"
        );
    }

    #[test]
    fn noteoff_short() {
        assert_eq!(format(&[0x80, 60, 48], Style::Short).unwrap(), "Noff C4 p");
    }

    // ── ControlChange ────────────────────────────────────────────────────────

    #[test]
    fn cc_long() {
        assert_eq!(
            format(&[0xB0, 7, 64], Style::Long).unwrap(),
            "ControlChange 7 64"
        );
    }

    #[test]
    fn cc_short() {
        assert_eq!(format(&[0xB0, 7, 64], Style::Short).unwrap(), "CC 7 64");
    }

    // ── ProgramChange ────────────────────────────────────────────────────────

    #[test]
    fn pc_long() {
        assert_eq!(
            format(&[0xC0, 40], Style::Long).unwrap(),
            "ProgramChange 40"
        );
    }

    #[test]
    fn pc_short() {
        assert_eq!(format(&[0xC0, 40], Style::Short).unwrap(), "PC 40");
    }

    // ── ChannelPressure ──────────────────────────────────────────────────────

    #[test]
    fn cp_long() {
        assert_eq!(
            format(&[0xD0, 64], Style::Long).unwrap(),
            "ChannelPressure 64"
        );
    }

    #[test]
    fn cp_short() {
        assert_eq!(format(&[0xD0, 64], Style::Short).unwrap(), "CP 64");
    }

    // ── PolyPressure ─────────────────────────────────────────────────────────

    #[test]
    fn pp_long() {
        assert_eq!(
            format(&[0xA0, 60, 64], Style::Long).unwrap(),
            "PolyPressure C4 64"
        );
    }

    #[test]
    fn pp_short() {
        assert_eq!(format(&[0xA0, 60, 64], Style::Short).unwrap(), "PP C4 64");
    }

    // ── PitchBend ────────────────────────────────────────────────────────────

    #[test]
    fn pb_long() {
        assert_eq!(
            format(&[0xE0, 0, 64], Style::Long).unwrap(),
            "PitchBend 0 64"
        );
    }

    #[test]
    fn pb_short() {
        assert_eq!(format(&[0xE0, 0, 64], Style::Short).unwrap(), "PB 0 64");
    }

    // ── SysEx ────────────────────────────────────────────────────────────────

    #[test]
    fn sysex_formats() {
        assert_eq!(
            format(&[0xF0, 0x41, 0x10, 0xF7], Style::Long).unwrap(),
            "SysEx 0x41 0x10 0xF7"
        );
    }

    #[test]
    fn sysex_style_has_no_effect() {
        let long = format(&[0xF0, 0x41, 0xF7], Style::Long).unwrap();
        let short = format(&[0xF0, 0x41, 0xF7], Style::Short).unwrap();
        assert_eq!(long, short);
    }

    // ── Error cases ──────────────────────────────────────────────────────────

    #[test]
    fn empty_input() {
        assert!(matches!(
            format(&[], Style::Long),
            Err(FormatError::EmptyInput)
        ));
    }

    #[test]
    fn unknown_status() {
        assert!(matches!(
            format(&[0x00], Style::Long),
            Err(FormatError::UnknownStatus(_))
        ));
    }

    #[test]
    fn truncated_noteon() {
        assert!(matches!(
            format(&[0x90, 60], Style::Long),
            Err(FormatError::TruncatedMessage { .. })
        ));
    }

    #[test]
    fn truncated_sysex() {
        assert!(matches!(
            format(&[0xF0], Style::Long),
            Err(FormatError::TruncatedMessage { .. })
        ));
    }

    // ── Round-trip ───────────────────────────────────────────────────────────

    #[test]
    fn roundtrip_noteon() {
        let bytes = vec![0x90u8, 60, 80];
        let s = format(&bytes, Style::Long).unwrap();
        let back = crate::parse(&s).unwrap();
        assert_eq!(back, bytes);
    }

    #[test]
    fn roundtrip_noteoff() {
        let bytes = vec![0x80u8, 60, 48];
        let s = format(&bytes, Style::Long).unwrap();
        let back = crate::parse(&s).unwrap();
        assert_eq!(back, bytes);
    }

    #[test]
    fn roundtrip_cc() {
        let bytes = vec![0xB0u8, 7, 64];
        let s = format(&bytes, Style::Long).unwrap();
        let back = crate::parse(&s).unwrap();
        assert_eq!(back, bytes);
    }

    #[test]
    fn roundtrip_pc() {
        let bytes = vec![0xC0u8, 40];
        let s = format(&bytes, Style::Long).unwrap();
        let back = crate::parse(&s).unwrap();
        assert_eq!(back, bytes);
    }

    #[test]
    fn roundtrip_sysex() {
        let bytes = vec![0xF0u8, 0x41, 0x10, 0xF7];
        let s = format(&bytes, Style::Long).unwrap();
        let back = crate::parse(&s).unwrap();
        assert_eq!(back, bytes);
    }

    #[test]
    fn roundtrip_channel_2() {
        let bytes = vec![0x91u8, 60, 80];
        let s = format(&bytes, Style::Long).unwrap();
        let back = crate::parse(&s).unwrap();
        assert_eq!(back, bytes);
    }
}

