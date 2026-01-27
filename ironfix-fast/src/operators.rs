/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! FAST field operators.
//!
//! Operators define how field values are encoded and decoded relative to
//! previous values in the dictionary.

use serde::{Deserialize, Serialize};

/// FAST field operator types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Operator {
    /// No operator - value is always present in stream.
    #[default]
    None,
    /// Constant - value is never in stream, always uses initial value.
    Constant,
    /// Default - if absent, use initial value.
    Default,
    /// Copy - if absent, use previous value from dictionary.
    Copy,
    /// Increment - if absent, increment previous value by 1.
    Increment,
    /// Delta - value in stream is delta from previous value.
    Delta,
    /// Tail - value in stream replaces tail of previous value.
    Tail,
}

impl Operator {
    /// Returns true if this operator uses the dictionary.
    #[must_use]
    pub const fn uses_dictionary(&self) -> bool {
        matches!(
            self,
            Self::Copy | Self::Increment | Self::Delta | Self::Tail
        )
    }

    /// Returns true if this operator requires a presence map bit.
    #[must_use]
    pub const fn requires_pmap(&self) -> bool {
        matches!(
            self,
            Self::None | Self::Copy | Self::Increment | Self::Delta | Self::Tail
        )
    }

    /// Returns true if the value can be absent from the stream.
    #[must_use]
    pub const fn can_be_absent(&self) -> bool {
        matches!(
            self,
            Self::Default | Self::Copy | Self::Increment | Self::Delta | Self::Tail
        )
    }
}

/// Dictionary scope for operator state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DictionaryScope {
    /// Global dictionary shared across all templates.
    #[default]
    Global,
    /// Template-specific dictionary.
    Template,
    /// Type-specific dictionary.
    Type,
}

/// State for a dictionary entry.
#[derive(Debug, Clone, Default)]
pub enum DictionaryValue {
    /// No value has been set.
    #[default]
    Undefined,
    /// Value is explicitly empty/null.
    Empty,
    /// Integer value.
    Int(i64),
    /// Unsigned integer value.
    UInt(u64),
    /// String value.
    String(String),
    /// Byte sequence value.
    Bytes(Vec<u8>),
    /// Decimal value (mantissa, exponent).
    Decimal(i64, i32),
}

impl DictionaryValue {
    /// Returns true if the value is undefined.
    #[must_use]
    pub const fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    /// Returns true if the value is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns the value as an i64, if applicable.
    #[must_use]
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the value as a u64, if applicable.
    #[must_use]
    pub const fn as_u64(&self) -> Option<u64> {
        match self {
            Self::UInt(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the value as a string, if applicable.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_uses_dictionary() {
        assert!(!Operator::None.uses_dictionary());
        assert!(!Operator::Constant.uses_dictionary());
        assert!(!Operator::Default.uses_dictionary());
        assert!(Operator::Copy.uses_dictionary());
        assert!(Operator::Increment.uses_dictionary());
        assert!(Operator::Delta.uses_dictionary());
        assert!(Operator::Tail.uses_dictionary());
    }

    #[test]
    fn test_operator_requires_pmap() {
        assert!(Operator::None.requires_pmap());
        assert!(!Operator::Constant.requires_pmap());
        assert!(!Operator::Default.requires_pmap());
        assert!(Operator::Copy.requires_pmap());
    }

    #[test]
    fn test_dictionary_value() {
        let undefined = DictionaryValue::Undefined;
        assert!(undefined.is_undefined());

        let int_val = DictionaryValue::Int(42);
        assert_eq!(int_val.as_i64(), Some(42));

        let str_val = DictionaryValue::String("test".to_string());
        assert_eq!(str_val.as_str(), Some("test"));
    }
}
