/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! FAST protocol encoder.
//!
//! This module provides encoding of values using FAST stop-bit encoding.

use crate::operators::DictionaryValue;
use std::collections::HashMap;

/// FAST protocol encoder.
#[derive(Debug)]
pub struct FastEncoder {
    /// Output buffer.
    buffer: Vec<u8>,
    /// Global dictionary for operator state.
    global_dict: HashMap<String, DictionaryValue>,
    /// Template-specific dictionaries.
    template_dicts: HashMap<u32, HashMap<String, DictionaryValue>>,
}

impl FastEncoder {
    /// Creates a new FAST encoder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            global_dict: HashMap::new(),
            template_dicts: HashMap::new(),
        }
    }

    /// Creates a new encoder with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            global_dict: HashMap::new(),
            template_dicts: HashMap::new(),
        }
    }

    /// Encodes an unsigned integer using stop-bit encoding.
    ///
    /// # Arguments
    /// * `value` - The value to encode
    pub fn encode_uint(&mut self, value: u64) {
        if value == 0 {
            self.buffer.push(0x80);
            return;
        }

        let mut bytes = Vec::new();
        let mut v = value;

        while v > 0 {
            bytes.push((v & 0x7F) as u8);
            v >>= 7;
        }

        bytes.reverse();

        // Set stop bit on last byte
        if let Some(last) = bytes.last_mut() {
            *last |= 0x80;
        }

        self.buffer.extend(bytes);
    }

    /// Encodes a signed integer using stop-bit encoding.
    ///
    /// # Arguments
    /// * `value` - The value to encode
    pub fn encode_int(&mut self, value: i64) {
        if (0..64).contains(&value) {
            self.buffer.push((value as u8) | 0x80);
            return;
        }

        if (-64..0).contains(&value) {
            self.buffer.push((value as u8) | 0x80);
            return;
        }

        let mut bytes = Vec::new();
        let mut v = value;
        let negative = value < 0;

        loop {
            bytes.push((v & 0x7F) as u8);
            v >>= 7;

            if (negative && v == -1 && (bytes.last().unwrap() & 0x40) != 0)
                || (!negative && v == 0 && (bytes.last().unwrap() & 0x40) == 0)
            {
                break;
            }

            if v == 0 && !negative {
                break;
            }
            if v == -1 && negative {
                break;
            }
        }

        bytes.reverse();

        if let Some(last) = bytes.last_mut() {
            *last |= 0x80;
        }

        self.buffer.extend(bytes);
    }

    /// Encodes an ASCII string using stop-bit encoding.
    ///
    /// # Arguments
    /// * `value` - The string to encode
    pub fn encode_ascii(&mut self, value: &str) {
        let bytes = value.as_bytes();

        if bytes.is_empty() {
            self.buffer.push(0x80);
            return;
        }

        for (i, &b) in bytes.iter().enumerate() {
            if i == bytes.len() - 1 {
                self.buffer.push(b | 0x80);
            } else {
                self.buffer.push(b & 0x7F);
            }
        }
    }

    /// Encodes a byte vector with length prefix.
    ///
    /// # Arguments
    /// * `value` - The bytes to encode
    pub fn encode_bytes(&mut self, value: &[u8]) {
        self.encode_uint(value.len() as u64);
        self.buffer.extend_from_slice(value);
    }

    /// Encodes a nullable unsigned integer.
    ///
    /// # Arguments
    /// * `value` - The optional value to encode
    pub fn encode_nullable_uint(&mut self, value: Option<u64>) {
        match value {
            Some(v) => self.encode_uint(v + 1),
            None => self.buffer.push(0x80),
        }
    }

    /// Returns the encoded bytes.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }

    /// Returns a reference to the current buffer.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Returns the current buffer length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clears the buffer for reuse.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Resets the encoder including dictionaries.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.global_dict.clear();
        self.template_dicts.clear();
    }

    /// Gets a value from the global dictionary.
    #[must_use]
    pub fn get_global(&self, key: &str) -> Option<&DictionaryValue> {
        self.global_dict.get(key)
    }

    /// Sets a value in the global dictionary.
    pub fn set_global(&mut self, key: impl Into<String>, value: DictionaryValue) {
        self.global_dict.insert(key.into(), value);
    }
}

impl Default for FastEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_uint_zero() {
        let mut encoder = FastEncoder::new();
        encoder.encode_uint(0);
        assert_eq!(encoder.finish(), vec![0x80]);
    }

    #[test]
    fn test_encode_uint_one() {
        let mut encoder = FastEncoder::new();
        encoder.encode_uint(1);
        assert_eq!(encoder.finish(), vec![0x81]);
    }

    #[test]
    fn test_encode_uint_larger() {
        let mut encoder = FastEncoder::new();
        encoder.encode_uint(942);
        let bytes = encoder.finish();
        // 942 = 7 * 128 + 46, so first byte is 7, second is 46 | 0x80 = 0xAE
        assert_eq!(bytes, vec![0x07, 0xAE]);
    }

    #[test]
    fn test_encode_ascii() {
        let mut encoder = FastEncoder::new();
        encoder.encode_ascii("Hi!");
        let bytes = encoder.finish();
        assert_eq!(bytes, vec![b'H', b'i', b'!' | 0x80]);
    }

    #[test]
    fn test_encode_ascii_empty() {
        let mut encoder = FastEncoder::new();
        encoder.encode_ascii("");
        assert_eq!(encoder.finish(), vec![0x80]);
    }

    #[test]
    fn test_encode_bytes() {
        let mut encoder = FastEncoder::new();
        encoder.encode_bytes(&[1, 2, 3]);
        let bytes = encoder.finish();
        assert_eq!(bytes, vec![0x83, 1, 2, 3]);
    }

    #[test]
    fn test_encoder_clear() {
        let mut encoder = FastEncoder::new();
        encoder.encode_uint(42);
        assert!(!encoder.is_empty());

        encoder.clear();
        assert!(encoder.is_empty());
    }
}
