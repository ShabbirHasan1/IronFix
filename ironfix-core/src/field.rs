/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! Field types and traits for FIX protocol messages.
//!
//! This module provides:
//! - [`FieldTag`]: Type-safe wrapper for FIX field tag numbers
//! - [`FieldRef`]: Zero-copy reference to a field within a message buffer
//! - [`FieldValue`]: Enumeration of possible field value types
//! - [`FixField`]: Trait for typed field access

use crate::error::DecodeError;
use bytes::Bytes;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// FIX field tag number.
///
/// Tags are positive integers that identify fields within a FIX message.
/// Standard tags are defined in the FIX specification (1-5000 range),
/// while user-defined tags use the 5001+ range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct FieldTag(u32);

impl FieldTag {
    /// Creates a new field tag.
    ///
    /// # Arguments
    /// * `tag` - The tag number (must be > 0)
    #[inline]
    #[must_use]
    pub const fn new(tag: u32) -> Self {
        Self(tag)
    }

    /// Returns the raw tag number.
    #[inline]
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Returns true if this is a standard FIX tag (1-5000).
    #[inline]
    #[must_use]
    pub const fn is_standard(self) -> bool {
        self.0 >= 1 && self.0 <= 5000
    }

    /// Returns true if this is a user-defined tag (5001+).
    #[inline]
    #[must_use]
    pub const fn is_user_defined(self) -> bool {
        self.0 > 5000
    }
}

impl From<u32> for FieldTag {
    fn from(tag: u32) -> Self {
        Self(tag)
    }
}

impl From<FieldTag> for u32 {
    fn from(tag: FieldTag) -> Self {
        tag.0
    }
}

impl fmt::Display for FieldTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Zero-copy reference to a field within a FIX message buffer.
///
/// This struct holds references to the original message buffer,
/// avoiding allocation during parsing.
#[derive(Debug, Clone, Copy)]
pub struct FieldRef<'a> {
    /// The field tag number.
    pub tag: u32,
    /// Reference to the field value bytes (without delimiters).
    pub value: &'a [u8],
}

impl<'a> FieldRef<'a> {
    /// Creates a new field reference.
    ///
    /// # Arguments
    /// * `tag` - The field tag number
    /// * `value` - Reference to the value bytes
    #[inline]
    #[must_use]
    pub const fn new(tag: u32, value: &'a [u8]) -> Self {
        Self { tag, value }
    }

    /// Returns the field tag.
    #[inline]
    #[must_use]
    pub const fn tag(&self) -> FieldTag {
        FieldTag(self.tag)
    }

    /// Returns the value as a string slice.
    ///
    /// # Errors
    /// Returns `DecodeError::InvalidUtf8` if the value is not valid UTF-8.
    pub fn as_str(&self) -> Result<&'a str, DecodeError> {
        std::str::from_utf8(self.value).map_err(DecodeError::from)
    }

    /// Returns the value as an owned String.
    ///
    /// # Errors
    /// Returns `DecodeError::InvalidUtf8` if the value is not valid UTF-8.
    pub fn to_string(&self) -> Result<String, DecodeError> {
        self.as_str().map(String::from)
    }

    /// Parses the value as the specified type.
    ///
    /// # Errors
    /// Returns `DecodeError::InvalidFieldValue` if parsing fails.
    pub fn parse<T: FromStr>(&self) -> Result<T, DecodeError> {
        let s = self.as_str()?;
        s.parse().map_err(|_| DecodeError::InvalidFieldValue {
            tag: self.tag,
            reason: format!("failed to parse '{}' as {}", s, std::any::type_name::<T>()),
        })
    }

    /// Returns the value as a u64.
    ///
    /// # Errors
    /// Returns `DecodeError::InvalidFieldValue` if the value is not a valid integer.
    pub fn as_u64(&self) -> Result<u64, DecodeError> {
        self.parse()
    }

    /// Returns the value as an i64.
    ///
    /// # Errors
    /// Returns `DecodeError::InvalidFieldValue` if the value is not a valid integer.
    pub fn as_i64(&self) -> Result<i64, DecodeError> {
        self.parse()
    }

    /// Returns the value as a Decimal.
    ///
    /// # Errors
    /// Returns `DecodeError::InvalidFieldValue` if the value is not a valid decimal.
    pub fn as_decimal(&self) -> Result<Decimal, DecodeError> {
        self.parse()
    }

    /// Returns the value as a bool (FIX uses 'Y'/'N').
    ///
    /// # Errors
    /// Returns `DecodeError::InvalidFieldValue` if the value is not 'Y' or 'N'.
    pub fn as_bool(&self) -> Result<bool, DecodeError> {
        match self.value {
            b"Y" => Ok(true),
            b"N" => Ok(false),
            _ => Err(DecodeError::InvalidFieldValue {
                tag: self.tag,
                reason: "expected 'Y' or 'N'".to_string(),
            }),
        }
    }

    /// Returns the value as a single character.
    ///
    /// # Errors
    /// Returns `DecodeError::InvalidFieldValue` if the value is not a single ASCII character.
    pub fn as_char(&self) -> Result<char, DecodeError> {
        if self.value.len() == 1 && self.value[0].is_ascii() {
            Ok(self.value[0] as char)
        } else {
            Err(DecodeError::InvalidFieldValue {
                tag: self.tag,
                reason: "expected single ASCII character".to_string(),
            })
        }
    }

    /// Returns the raw bytes of the value.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.value
    }

    /// Returns the length of the value in bytes.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.value.len()
    }

    /// Returns true if the value is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

/// Enumeration of possible FIX field value types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldValue {
    /// String value.
    String(String),
    /// Integer value.
    Int(i64),
    /// Unsigned integer value.
    UInt(u64),
    /// Decimal/float value.
    Decimal(Decimal),
    /// Boolean value (Y/N).
    Bool(bool),
    /// Single character value.
    Char(char),
    /// Raw bytes (for data fields).
    Data(Bytes),
}

impl FieldValue {
    /// Returns the value as a string, if it is a String variant.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the value as an i64, if it is an Int variant.
    #[must_use]
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the value as a u64, if it is a UInt variant.
    #[must_use]
    pub const fn as_u64(&self) -> Option<u64> {
        match self {
            Self::UInt(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the value as a Decimal, if it is a Decimal variant.
    #[must_use]
    pub const fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Self::Decimal(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the value as a bool, if it is a Bool variant.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the value as a char, if it is a Char variant.
    #[must_use]
    pub const fn as_char(&self) -> Option<char> {
        match self {
            Self::Char(v) => Some(*v),
            _ => None,
        }
    }
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Int(v) => write!(f, "{}", v),
            Self::UInt(v) => write!(f, "{}", v),
            Self::Decimal(v) => write!(f, "{}", v),
            Self::Bool(v) => write!(f, "{}", if *v { "Y" } else { "N" }),
            Self::Char(c) => write!(f, "{}", c),
            Self::Data(d) => write!(f, "<{} bytes>", d.len()),
        }
    }
}

/// Trait for typed FIX field access.
///
/// This trait is implemented by generated field types to provide
/// type-safe access to field values.
pub trait FixField: Sized {
    /// The tag number for this field.
    const TAG: u32;

    /// The Rust type for this field's value.
    type Value;

    /// Decodes the field value from a byte slice.
    ///
    /// # Arguments
    /// * `bytes` - The raw bytes of the field value
    ///
    /// # Errors
    /// Returns `DecodeError` if the value cannot be decoded.
    fn decode(bytes: &[u8]) -> Result<Self::Value, DecodeError>;

    /// Encodes the field value to bytes.
    ///
    /// # Arguments
    /// * `value` - The value to encode
    /// * `buf` - The buffer to write to
    fn encode(value: &Self::Value, buf: &mut Vec<u8>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_tag() {
        let tag = FieldTag::new(35);
        assert_eq!(tag.value(), 35);
        assert!(tag.is_standard());
        assert!(!tag.is_user_defined());

        let user_tag = FieldTag::new(5001);
        assert!(!user_tag.is_standard());
        assert!(user_tag.is_user_defined());
    }

    #[test]
    fn test_field_ref_as_str() {
        let field = FieldRef::new(11, b"ORDER123");
        assert_eq!(field.as_str().unwrap(), "ORDER123");
    }

    #[test]
    fn test_field_ref_as_u64() {
        let field = FieldRef::new(34, b"12345");
        assert_eq!(field.as_u64().unwrap(), 12345);
    }

    #[test]
    fn test_field_ref_as_bool() {
        let yes = FieldRef::new(141, b"Y");
        let no = FieldRef::new(141, b"N");
        assert!(yes.as_bool().unwrap());
        assert!(!no.as_bool().unwrap());
    }

    #[test]
    fn test_field_ref_as_char() {
        let field = FieldRef::new(54, b"1");
        assert_eq!(field.as_char().unwrap(), '1');
    }

    #[test]
    fn test_field_ref_invalid_utf8() {
        let field = FieldRef::new(1, &[0xFF, 0xFE]);
        assert!(field.as_str().is_err());
    }

    #[test]
    fn test_field_value_display() {
        assert_eq!(FieldValue::String("test".to_string()).to_string(), "test");
        assert_eq!(FieldValue::Int(42).to_string(), "42");
        assert_eq!(FieldValue::Bool(true).to_string(), "Y");
        assert_eq!(FieldValue::Bool(false).to_string(), "N");
    }
}
