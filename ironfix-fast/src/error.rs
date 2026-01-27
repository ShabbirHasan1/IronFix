/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! FAST protocol error types.

use thiserror::Error;

/// Errors that can occur during FAST encoding/decoding.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FastError {
    /// Unexpected end of input.
    #[error("unexpected end of input")]
    UnexpectedEof,

    /// Unknown template ID.
    #[error("unknown template id: {0}")]
    UnknownTemplate(u32),

    /// Invalid presence map.
    #[error("invalid presence map")]
    InvalidPresenceMap,

    /// Integer overflow during decoding.
    #[error("integer overflow")]
    IntegerOverflow,

    /// Invalid string encoding.
    #[error("invalid string encoding")]
    InvalidString,

    /// Invalid decimal encoding.
    #[error("invalid decimal: exponent={exponent}, mantissa={mantissa}")]
    InvalidDecimal {
        /// Decimal exponent.
        exponent: i32,
        /// Decimal mantissa.
        mantissa: i64,
    },

    /// Missing mandatory field.
    #[error("missing mandatory field: {name}")]
    MissingMandatoryField {
        /// Field name.
        name: String,
    },

    /// Invalid operator application.
    #[error("invalid operator: {0}")]
    InvalidOperator(String),

    /// Dictionary entry not found.
    #[error("dictionary entry not found: {key}")]
    DictionaryEntryNotFound {
        /// Dictionary key.
        key: String,
    },

    /// Sequence length mismatch.
    #[error("sequence length mismatch: expected {expected}, got {actual}")]
    SequenceLengthMismatch {
        /// Expected length.
        expected: u32,
        /// Actual length.
        actual: u32,
    },
}
