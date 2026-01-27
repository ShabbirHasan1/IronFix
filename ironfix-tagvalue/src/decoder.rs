/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! Zero-copy FIX message decoder.
//!
//! This module provides a high-performance decoder that parses FIX messages
//! without allocating memory for field values. Field values are returned as
//! references to the original buffer.

use crate::checksum::{calculate_checksum, parse_checksum};
use ironfix_core::error::DecodeError;
use ironfix_core::field::FieldRef;
use ironfix_core::message::{MsgType, RawMessage};
use memchr::memchr;
use smallvec::SmallVec;

/// SOH (Start of Header) delimiter used in FIX messages.
pub const SOH: u8 = 0x01;

/// Equals sign delimiter between tag and value.
pub const EQUALS: u8 = b'=';

/// Zero-copy FIX message decoder.
///
/// The decoder parses FIX messages from a byte buffer, extracting fields
/// as references to the original data without copying.
#[derive(Debug)]
pub struct Decoder<'a> {
    /// Input buffer.
    input: &'a [u8],
    /// Current position in the buffer.
    offset: usize,
    /// Whether to validate checksums.
    validate_checksum: bool,
}

impl<'a> Decoder<'a> {
    /// Creates a new decoder for the given input buffer.
    ///
    /// # Arguments
    /// * `input` - The FIX message bytes to decode
    #[inline]
    #[must_use]
    pub const fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            offset: 0,
            validate_checksum: true,
        }
    }

    /// Sets whether to validate checksums during decoding.
    ///
    /// # Arguments
    /// * `validate` - Whether to validate checksums
    #[inline]
    #[must_use]
    pub const fn with_checksum_validation(mut self, validate: bool) -> Self {
        self.validate_checksum = validate;
        self
    }

    /// Decodes a complete FIX message from the buffer.
    ///
    /// # Returns
    /// A `RawMessage` containing zero-copy references to the parsed fields.
    ///
    /// # Errors
    /// Returns `DecodeError` if the message is malformed or incomplete.
    pub fn decode(&mut self) -> Result<RawMessage<'a>, DecodeError> {
        let start_offset = self.offset;

        // Parse BeginString (tag 8)
        let begin_string_field = self.next_field().ok_or(DecodeError::Incomplete)?;
        if begin_string_field.tag != 8 {
            return Err(DecodeError::InvalidBeginString);
        }
        let begin_string_start =
            begin_string_field.value.as_ptr() as usize - self.input.as_ptr() as usize;
        let begin_string_end = begin_string_start + begin_string_field.value.len();
        let begin_string = begin_string_start..begin_string_end;

        // Parse BodyLength (tag 9)
        let body_length_field = self.next_field().ok_or(DecodeError::MissingBodyLength)?;
        if body_length_field.tag != 9 {
            return Err(DecodeError::MissingBodyLength);
        }
        let body_length: usize = body_length_field
            .as_str()?
            .parse()
            .map_err(|_| DecodeError::InvalidBodyLength)?;

        // Record body start position
        let body_start = self.offset;

        // Parse MsgType (tag 35) - should be first field in body
        let msg_type_field = self.next_field().ok_or(DecodeError::MissingMsgType)?;
        if msg_type_field.tag != 35 {
            return Err(DecodeError::MissingMsgType);
        }
        let msg_type: MsgType = msg_type_field.as_str()?.parse().unwrap();

        // Collect all fields
        let mut fields: SmallVec<[FieldRef<'a>; 32]> = SmallVec::new();
        fields.push(begin_string_field);
        fields.push(body_length_field);
        fields.push(msg_type_field);

        // Parse remaining fields until checksum
        let mut checksum_field: Option<FieldRef<'a>> = None;
        while let Some(field) = self.next_field() {
            if field.tag == 10 {
                checksum_field = Some(field);
                break;
            }
            fields.push(field);
        }

        // Validate checksum if enabled
        if self.validate_checksum {
            let checksum_ref = checksum_field.ok_or(DecodeError::Incomplete)?;
            let declared = parse_checksum(checksum_ref.value).ok_or_else(|| {
                DecodeError::InvalidFieldValue {
                    tag: 10,
                    reason: "invalid checksum format".to_string(),
                }
            })?;

            // Calculate checksum of everything before the checksum field
            let checksum_start =
                checksum_ref.value.as_ptr() as usize - self.input.as_ptr() as usize - 3; // "10="
            let calculated = calculate_checksum(&self.input[start_offset..checksum_start]);

            if calculated != declared {
                return Err(DecodeError::ChecksumMismatch {
                    calculated,
                    declared,
                });
            }
        }

        let body_end = body_start + body_length;
        let body = body_start..body_end;

        Ok(RawMessage::new(
            &self.input[start_offset..self.offset],
            begin_string,
            body,
            msg_type,
            fields,
        ))
    }

    /// Parses the next field from the buffer.
    ///
    /// # Returns
    /// The next field, or `None` if the buffer is exhausted.
    #[inline]
    pub fn next_field(&mut self) -> Option<FieldRef<'a>> {
        if self.offset >= self.input.len() {
            return None;
        }

        let remaining = &self.input[self.offset..];

        // Find '=' delimiter using SIMD-accelerated search
        let eq_pos = memchr(EQUALS, remaining)?;
        let tag_bytes = &remaining[..eq_pos];

        // Parse tag number
        let tag = parse_tag(tag_bytes)?;

        // Find SOH delimiter
        let value_start = eq_pos + 1;
        let soh_pos = memchr(SOH, &remaining[value_start..])?;
        let value = &remaining[value_start..value_start + soh_pos];

        self.offset += value_start + soh_pos + 1;

        Some(FieldRef::new(tag, value))
    }

    /// Returns the current offset in the buffer.
    #[inline]
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the remaining bytes in the buffer.
    #[inline]
    #[must_use]
    pub fn remaining(&self) -> &'a [u8] {
        &self.input[self.offset..]
    }

    /// Returns true if the buffer has been fully consumed.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.offset >= self.input.len()
    }

    /// Resets the decoder to the beginning of the buffer.
    #[inline]
    pub fn reset(&mut self) {
        self.offset = 0;
    }
}

/// Parses a tag number from ASCII bytes.
///
/// # Arguments
/// * `bytes` - The ASCII bytes representing the tag number
///
/// # Returns
/// The parsed tag number, or `None` if invalid.
#[inline]
fn parse_tag(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() || bytes.len() > 10 {
        return None;
    }

    let mut result: u32 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag() {
        assert_eq!(parse_tag(b"8"), Some(8));
        assert_eq!(parse_tag(b"35"), Some(35));
        assert_eq!(parse_tag(b"12345"), Some(12345));
        assert_eq!(parse_tag(b""), None);
        assert_eq!(parse_tag(b"abc"), None);
        assert_eq!(parse_tag(b"12a"), None);
    }

    #[test]
    fn test_next_field() {
        let input = b"8=FIX.4.4\x019=5\x0135=0\x01";
        let mut decoder = Decoder::new(input);

        let field1 = decoder.next_field().unwrap();
        assert_eq!(field1.tag, 8);
        assert_eq!(field1.as_str().unwrap(), "FIX.4.4");

        let field2 = decoder.next_field().unwrap();
        assert_eq!(field2.tag, 9);
        assert_eq!(field2.as_str().unwrap(), "5");

        let field3 = decoder.next_field().unwrap();
        assert_eq!(field3.tag, 35);
        assert_eq!(field3.as_str().unwrap(), "0");

        assert!(decoder.next_field().is_none());
    }

    #[test]
    fn test_decoder_empty() {
        let mut decoder = Decoder::new(b"");
        assert!(decoder.next_field().is_none());
        assert!(decoder.is_empty());
    }

    #[test]
    fn test_decoder_incomplete() {
        let input = b"8=FIX.4.4";
        let mut decoder = Decoder::new(input);
        assert!(decoder.next_field().is_none());
    }
}
