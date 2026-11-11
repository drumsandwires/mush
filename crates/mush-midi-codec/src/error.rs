use thiserror::Error;

/// Errors produced by [`crate::parse`].
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ParseError {
    #[error("empty input")]
    EmptyInput,

    #[error("unknown status symbol: '{0}'")]
    UnknownStatus(String),

    #[error("unknown note name: '{0}'")]
    UnknownNote(String),

    #[error("unknown velocity symbol: '{0}'")]
    UnknownVelocity(String),

    #[error("invalid channel '{0}': must be 1–16")]
    InvalidChannel(u8),

    #[error("byte value {0} out of range")]
    ByteOutOfRange(i64),

    #[error("invalid numeric literal: '{0}'")]
    InvalidNumericLiteral(String),

    #[error("'{status}' expects {expected} data byte(s), got {got}")]
    WrongArgumentCount {
        status:   String,
        expected: usize,
        got:      usize,
    },

    #[error("SysEx message missing 0xF7 terminator")]
    MissingSysExTerminator,

    #[error("unexpected trailing input: '{0}'")]
    TrailingInput(String),
}

/// Errors produced by [`crate::format`].
#[derive(Debug, Clone, PartialEq, Error)]
pub enum FormatError {
    #[error("empty input")]
    EmptyInput,

    #[error("unknown status byte: 0x{0:02X}")]
    UnknownStatus(u8),

    #[error("message truncated: status 0x{status:02X} expects {expected} byte(s), got {got}")]
    TruncatedMessage {
        status:   u8,
        expected: usize,
        got:      usize,
    },
}