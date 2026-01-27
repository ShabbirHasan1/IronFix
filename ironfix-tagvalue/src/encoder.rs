/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! FIX message encoder.
//!
//! This module provides an encoder for building FIX messages in the
//! standard tag=value format.

use crate::checksum::{calculate_checksum, format_checksum};
use bytes::{BufMut, BytesMut};

/// SOH (Start of Header) delimiter used in FIX messages.
pub const SOH: u8 = 0x01;

/// FIX message encoder.
///
/// The encoder builds FIX messages by appending fields in tag=value format.
/// It handles BeginString, BodyLength, and Checksum fields automatically.
#[derive(Debug)]
pub struct Encoder {
    /// Buffer for the message body (between BodyLength and Checksum).
    body: BytesMut,
    /// The BeginString value (e.g., "FIX.4.4").
    begin_string: &'static str,
}

impl Encoder {
    /// Creates a new encoder with the specified BeginString.
    ///
    /// # Arguments
    /// * `begin_string` - The FIX version string (e.g., "FIX.4.4")
    #[must_use]
    pub fn new(begin_string: &'static str) -> Self {
        Self {
            body: BytesMut::with_capacity(256),
            begin_string,
        }
    }

    /// Creates a new encoder with pre-allocated capacity.
    ///
    /// # Arguments
    /// * `begin_string` - The FIX version string
    /// * `capacity` - Initial buffer capacity in bytes
    #[must_use]
    pub fn with_capacity(begin_string: &'static str, capacity: usize) -> Self {
        Self {
            body: BytesMut::with_capacity(capacity),
            begin_string,
        }
    }

    /// Appends a field with a string value.
    ///
    /// # Arguments
    /// * `tag` - The field tag number
    /// * `value` - The field value
    #[inline]
    pub fn put_str(&mut self, tag: u32, value: &str) {
        self.put_raw(tag, value.as_bytes());
    }

    /// Appends a field with an integer value.
    ///
    /// # Arguments
    /// * `tag` - The field tag number
    /// * `value` - The field value
    #[inline]
    pub fn put_int(&mut self, tag: u32, value: i64) {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.put_raw(tag, s.as_bytes());
    }

    /// Appends a field with an unsigned integer value.
    ///
    /// # Arguments
    /// * `tag` - The field tag number
    /// * `value` - The field value
    #[inline]
    pub fn put_uint(&mut self, tag: u32, value: u64) {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.put_raw(tag, s.as_bytes());
    }

    /// Appends a field with a boolean value (Y/N).
    ///
    /// # Arguments
    /// * `tag` - The field tag number
    /// * `value` - The field value
    #[inline]
    pub fn put_bool(&mut self, tag: u32, value: bool) {
        self.put_raw(tag, if value { b"Y" } else { b"N" });
    }

    /// Appends a field with a single character value.
    ///
    /// # Arguments
    /// * `tag` - The field tag number
    /// * `value` - The field value
    #[inline]
    pub fn put_char(&mut self, tag: u32, value: char) {
        let mut buf = [0u8; 4];
        let s = value.encode_utf8(&mut buf);
        self.put_raw(tag, s.as_bytes());
    }

    /// Appends a field with raw bytes.
    ///
    /// # Arguments
    /// * `tag` - The field tag number
    /// * `value` - The field value bytes
    #[inline]
    pub fn put_raw(&mut self, tag: u32, value: &[u8]) {
        let mut tag_buf = itoa::Buffer::new();
        let tag_str = tag_buf.format(tag);

        self.body.put_slice(tag_str.as_bytes());
        self.body.put_u8(b'=');
        self.body.put_slice(value);
        self.body.put_u8(SOH);
    }

    /// Finalizes the message and returns the complete encoded bytes.
    ///
    /// This method:
    /// 1. Prepends BeginString (tag 8) and BodyLength (tag 9)
    /// 2. Appends Checksum (tag 10)
    ///
    /// # Returns
    /// The complete FIX message as bytes.
    #[must_use]
    pub fn finish(self) -> BytesMut {
        let body_len = self.body.len();

        // Build header: 8=BeginString|9=BodyLength|
        let mut header = BytesMut::with_capacity(32);
        header.put_slice(b"8=");
        header.put_slice(self.begin_string.as_bytes());
        header.put_u8(SOH);
        header.put_slice(b"9=");

        let mut len_buf = itoa::Buffer::new();
        let len_str = len_buf.format(body_len);
        header.put_slice(len_str.as_bytes());
        header.put_u8(SOH);

        // Combine header and body
        let mut message = BytesMut::with_capacity(header.len() + body_len + 8);
        message.put_slice(&header);
        message.put_slice(&self.body);

        // Calculate and append checksum
        let checksum = calculate_checksum(&message);
        let checksum_bytes = format_checksum(checksum);

        message.put_slice(b"10=");
        message.put_slice(&checksum_bytes);
        message.put_u8(SOH);

        message
    }

    /// Returns the current body length.
    #[inline]
    #[must_use]
    pub fn body_len(&self) -> usize {
        self.body.len()
    }

    /// Clears the encoder for reuse.
    #[inline]
    pub fn clear(&mut self) {
        self.body.clear();
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new("FIX.4.4")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_basic() {
        let mut encoder = Encoder::new("FIX.4.4");
        encoder.put_str(35, "0");

        let message = encoder.finish();
        let msg_str = String::from_utf8_lossy(&message);

        assert!(msg_str.starts_with("8=FIX.4.4\x01"));
        assert!(msg_str.contains("35=0\x01"));
        assert!(msg_str.contains("10="));
    }

    #[test]
    fn test_encoder_multiple_fields() {
        let mut encoder = Encoder::new("FIX.4.4");
        encoder.put_str(35, "D");
        encoder.put_str(49, "SENDER");
        encoder.put_str(56, "TARGET");
        encoder.put_uint(34, 1);

        let message = encoder.finish();
        let msg_str = String::from_utf8_lossy(&message);

        assert!(msg_str.contains("35=D\x01"));
        assert!(msg_str.contains("49=SENDER\x01"));
        assert!(msg_str.contains("56=TARGET\x01"));
        assert!(msg_str.contains("34=1\x01"));
    }

    #[test]
    fn test_encoder_bool() {
        let mut encoder = Encoder::new("FIX.4.4");
        encoder.put_bool(141, true);
        encoder.put_bool(142, false);

        let message = encoder.finish();
        let msg_str = String::from_utf8_lossy(&message);

        assert!(msg_str.contains("141=Y\x01"));
        assert!(msg_str.contains("142=N\x01"));
    }

    #[test]
    fn test_encoder_char() {
        let mut encoder = Encoder::new("FIX.4.4");
        encoder.put_char(54, '1');

        let message = encoder.finish();
        let msg_str = String::from_utf8_lossy(&message);

        assert!(msg_str.contains("54=1\x01"));
    }

    #[test]
    fn test_encoder_clear() {
        let mut encoder = Encoder::new("FIX.4.4");
        encoder.put_str(35, "0");
        assert!(encoder.body_len() > 0);

        encoder.clear();
        assert_eq!(encoder.body_len(), 0);
    }
}
