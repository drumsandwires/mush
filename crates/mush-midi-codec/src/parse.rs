use crate::error::ParseError;
use crate::note::note_from_str;
use crate::status::StatusKind;
use crate::velocity::velocity_from_str;

/// Parse a symbolic or numeric MIDI message string into raw wire bytes.
///
/// See the crate-level documentation for the full grammar.
pub fn parse(input: &str) -> Result<Vec<u8>, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let tokens: Vec<&str> = input.split_whitespace().collect();

    // ── Numeric three-byte form: `144 60 100` / `0x90 0x3C 0x64` ──────────
    if tokens.len() == 3 {
        if let (Ok(b0), Ok(b1), Ok(b2)) = (
            parse_numeric(tokens[0]),
            parse_numeric(tokens[1]),
            parse_numeric(tokens[2]),
        ) {
            return Ok(vec![b0 as u8, b1 as u8, b2 as u8]);
        }
    }

    // ── Symbolic form ───────────────────────────────────────────────────────
    let (status_sym, channel) = parse_status_token(tokens[0])?;

    if status_sym == StatusKind::SysEx {
        return parse_sysex(&tokens[1..]);
    }

    let data_tokens = &tokens[1..];
    let expected = status_sym.data_len();

    if data_tokens.len() != expected {
        return Err(ParseError::WrongArgumentCount {
            status: tokens[0].to_string(),
            expected,
            got: data_tokens.len(),
        });
    }

    let base = status_sym.base_status().unwrap();
    let status = base | (channel - 1); // channel is 1-indexed

    let mut bytes = vec![status];

    match status_sym {
        StatusKind::NoteOn | StatusKind::NoteOff | StatusKind::PolyPressure => {
            // data1 = note (name or numeric), data2 = velocity (symbol or numeric)
            bytes.push(parse_note_or_numeric(data_tokens[0])?);
            bytes.push(parse_vel_or_numeric(data_tokens[1])?);
        }
        StatusKind::ControlChange => {
            bytes.push(parse_data_byte(data_tokens[0])?);
            bytes.push(parse_data_byte(data_tokens[1])?);
        }
        StatusKind::ProgramChange | StatusKind::ChannelPressure => {
            bytes.push(parse_data_byte(data_tokens[0])?);
        }
        StatusKind::PitchBend => {
            bytes.push(parse_data_byte(data_tokens[0])?);
            bytes.push(parse_data_byte(data_tokens[1])?);
        }
        StatusKind::SysEx => unreachable!(),
    }

    Ok(bytes)
}

/// Parse `StatusKind` and channel from a token like `NoteOn` or `NoteOn/2`.
fn parse_status_token(token: &str) -> Result<(StatusKind, u8), ParseError> {
    let (sym_str, channel) = if let Some(slash) = token.find('/') {
        let sym = &token[..slash];
        let ch_str = &token[slash + 1..];
        let ch: u8 = ch_str.parse().map_err(|_| ParseError::InvalidChannel(0))?;
        if !(1..=16).contains(&ch) {
            return Err(ParseError::InvalidChannel(ch));
        }
        (sym, ch)
    } else {
        (token, 1u8)
    };

    let kind = StatusKind::from_str(sym_str)
        .ok_or_else(|| ParseError::UnknownStatus(sym_str.to_string()))?;

    Ok((kind, channel))
}

/// Parse a SysEx payload: one or more bytes ending with `0xF7`.
fn parse_sysex(tokens: &[&str]) -> Result<Vec<u8>, ParseError> {
    if tokens.is_empty() {
        return Err(ParseError::MissingSysExTerminator);
    }

    let last = *tokens.last().unwrap();
    let last_val = parse_numeric(last).map_err(|_| ParseError::MissingSysExTerminator)?;

    if last_val != 0xF7 {
        return Err(ParseError::MissingSysExTerminator);
    }

    let mut bytes = vec![0xF0u8];
    for token in tokens {
        let val = parse_numeric(token)?;
        if !(0..=255).contains(&val) {
            return Err(ParseError::ByteOutOfRange(val));
        }
        bytes.push(val as u8);
    }

    Ok(bytes)
}

/// Parse a data byte — numeric only (0–127).
fn parse_data_byte(s: &str) -> Result<u8, ParseError> {
    let val = parse_numeric(s)?;
    if !(0..=127).contains(&val) {
        return Err(ParseError::ByteOutOfRange(val));
    }
    Ok(val as u8)
}

/// Parse a note position — note name or numeric.
fn parse_note_or_numeric(s: &str) -> Result<u8, ParseError> {
    // If it starts with a letter A-G it's a note name.
    if s.chars()
        .next()
        .map(|c| matches!(c.to_ascii_uppercase(), 'A'..='G'))
        .unwrap_or(false)
    {
        return note_from_str(s);
    }
    parse_data_byte(s)
}

/// Parse a velocity position — dynamic symbol or numeric.
fn parse_vel_or_numeric(s: &str) -> Result<u8, ParseError> {
    // Try velocity symbol first; fall back to numeric.
    if let Ok(v) = velocity_from_str(s) {
        return Ok(v);
    }
    parse_data_byte(s)
}

/// Parse a numeric literal with optional prefix.
/// Returns i64 so callers can range-check for their context.
pub(crate) fn parse_numeric(s: &str) -> Result<i64, ParseError> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16).map_err(|_| ParseError::InvalidNumericLiteral(s.to_string()))
    } else if let Some(oct) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
        i64::from_str_radix(oct, 8).map_err(|_| ParseError::InvalidNumericLiteral(s.to_string()))
    } else if let Some(bin) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        i64::from_str_radix(bin, 2).map_err(|_| ParseError::InvalidNumericLiteral(s.to_string()))
    } else {
        s.parse::<i64>()
            .map_err(|_| ParseError::InvalidNumericLiteral(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Numeric form ─────────────────────────────────────────────────────────

    #[test]
    fn numeric_decimal() {
        assert_eq!(parse("144 60 100").unwrap(), vec![0x90, 60, 100]);
    }

    #[test]
    fn numeric_hex() {
        assert_eq!(parse("0x90 0x3C 0x64").unwrap(), vec![0x90, 0x3C, 0x64]);
    }

    #[test]
    fn numeric_octal() {
        assert_eq!(parse("0o220 0o74 0o144").unwrap(), vec![0x90, 0x3C, 0x64]);
    }

    #[test]
    fn numeric_binary() {
        assert_eq!(
            parse("0b10010000 0b00111100 0b01100100").unwrap(),
            vec![0x90, 0x3C, 0x64]
        );
    }

    // ── Symbolic NoteOn ──────────────────────────────────────────────────────

    #[test]
    fn noteon_long() {
        assert_eq!(parse("NoteOn C4 mf").unwrap(), vec![0x90, 60, 80]);
    }

    #[test]
    fn noteon_short() {
        assert_eq!(parse("Non C4 mf").unwrap(), vec![0x90, 60, 80]);
    }

    #[test]
    fn noteon_channel_2() {
        assert_eq!(parse("NoteOn/2 C4 mf").unwrap(), vec![0x91, 60, 80]);
    }

    #[test]
    fn noteon_channel_16() {
        assert_eq!(parse("NoteOn/16 C4 mf").unwrap(), vec![0x9F, 60, 80]);
    }

    #[test]
    fn noteon_numeric_note() {
        assert_eq!(parse("NoteOn 60 mf").unwrap(), vec![0x90, 60, 80]);
    }

    #[test]
    fn noteon_numeric_vel() {
        assert_eq!(parse("NoteOn C4 100").unwrap(), vec![0x90, 60, 100]);
    }

    #[test]
    fn noteon_all_numeric_data() {
        assert_eq!(parse("NoteOn 60 100").unwrap(), vec![0x90, 60, 100]);
    }

    #[test]
    fn noteon_hex_data() {
        assert_eq!(parse("NoteOn 0x3C 0x64").unwrap(), vec![0x90, 0x3C, 0x64]);
    }

    // ── Symbolic NoteOff ─────────────────────────────────────────────────────

    #[test]
    fn noteoff_long() {
        assert_eq!(parse("NoteOff C4 p").unwrap(), vec![0x80, 60, 48]);
    }

    #[test]
    fn noteoff_short() {
        assert_eq!(parse("Noff C4 p").unwrap(), vec![0x80, 60, 48]);
    }

    // ── ControlChange ────────────────────────────────────────────────────────

    #[test]
    fn cc_long() {
        assert_eq!(parse("ControlChange 7 64").unwrap(), vec![0xB0, 7, 64]);
    }

    #[test]
    fn cc_short() {
        assert_eq!(parse("CC 7 64").unwrap(), vec![0xB0, 7, 64]);
    }

    #[test]
    fn cc_channel() {
        assert_eq!(parse("CC/3 7 64").unwrap(), vec![0xB2, 7, 64]);
    }

    // ── ProgramChange ────────────────────────────────────────────────────────

    #[test]
    fn pc_long() {
        assert_eq!(parse("ProgramChange 40").unwrap(), vec![0xC0, 40]);
    }

    #[test]
    fn pc_short() {
        assert_eq!(parse("PC 40").unwrap(), vec![0xC0, 40]);
    }

    // ── ChannelPressure ──────────────────────────────────────────────────────

    #[test]
    fn cp_long() {
        assert_eq!(parse("ChannelPressure 64").unwrap(), vec![0xD0, 64]);
    }

    #[test]
    fn cp_short() {
        assert_eq!(parse("CP 64").unwrap(), vec![0xD0, 64]);
    }

    // ── PolyPressure ─────────────────────────────────────────────────────────

    #[test]
    fn pp_long() {
        assert_eq!(parse("PolyPressure C4 64").unwrap(), vec![0xA0, 60, 64]);
    }

    #[test]
    fn pp_short() {
        assert_eq!(parse("PP C4 64").unwrap(), vec![0xA0, 60, 64]);
    }

    // ── PitchBend ────────────────────────────────────────────────────────────

    #[test]
    fn pb_long() {
        assert_eq!(parse("PitchBend 0 64").unwrap(), vec![0xE0, 0, 64]);
    }

    #[test]
    fn pb_short() {
        assert_eq!(parse("PB 0 64").unwrap(), vec![0xE0, 0, 64]);
    }

    // ── SysEx ────────────────────────────────────────────────────────────────

    #[test]
    fn sysex_basic() {
        assert_eq!(
            parse("SysEx 0x41 0x10 0xF7").unwrap(),
            vec![0xF0, 0x41, 0x10, 0xF7]
        );
    }

    #[test]
    fn sysex_decimal() {
        assert_eq!(parse("SysEx 65 16 247").unwrap(), vec![0xF0, 65, 16, 247]);
    }

    #[test]
    fn sysex_missing_terminator() {
        assert!(matches!(
            parse("SysEx 0x41 0x10"),
            Err(ParseError::MissingSysExTerminator)
        ));
    }

    #[test]
    fn sysex_empty() {
        assert!(matches!(
            parse("SysEx"),
            Err(ParseError::MissingSysExTerminator)
        ));
    }

    // ── Error cases ──────────────────────────────────────────────────────────

    #[test]
    fn empty_input() {
        assert!(matches!(parse(""), Err(ParseError::EmptyInput)));
        assert!(matches!(parse("   "), Err(ParseError::EmptyInput)));
    }

    #[test]
    fn unknown_status() {
        assert!(matches!(
            parse("Blorp C4 mf"),
            Err(ParseError::UnknownStatus(_))
        ));
    }

    #[test]
    fn wrong_arg_count_too_few() {
        assert!(matches!(
            parse("NoteOn C4"),
            Err(ParseError::WrongArgumentCount { .. })
        ));
    }

    #[test]
    fn wrong_arg_count_too_many() {
        assert!(matches!(
            parse("PC 40 64"),
            Err(ParseError::WrongArgumentCount { .. })
        ));
    }

    #[test]
    fn invalid_channel_zero() {
        assert!(matches!(
            parse("NoteOn/0 C4 mf"),
            Err(ParseError::InvalidChannel(_))
        ));
    }

    #[test]
    fn invalid_channel_17() {
        assert!(matches!(
            parse("NoteOn/17 C4 mf"),
            Err(ParseError::InvalidChannel(_))
        ));
    }

    #[test]
    fn byte_out_of_range() {
        assert!(matches!(
            parse("NoteOn 128 mf"),
            Err(ParseError::ByteOutOfRange(_))
        ));
    }
}

