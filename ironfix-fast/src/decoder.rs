/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! FAST protocol decoder.
//!
//! This module provides decoding of FAST-encoded messages using stop-bit
//! encoding and presence maps.

use crate::error::FastError;
use crate::operators::DictionaryValue;
use crate::pmap::PresenceMap;
use std::collections::HashMap;

/// FAST protocol decoder.
#[derive(Debug)]
pub struct FastDecoder {
    /// Global dictionary for operator state.
    global_dict: HashMap<String, DictionaryValue>,
    /// Template-specific dictionaries.
    template_dicts: HashMap<u32, HashMap<String, DictionaryValue>>,
    /// Last used template ID.
    last_template_id: Option<u32>,
}

impl FastDecoder {
    /// Creates a new FAST decoder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            global_dict: HashMap::new(),
            template_dicts: HashMap::new(),
            last_template_id: None,
        }
    }

    /// Resets the decoder state.
    pub fn reset(&mut self) {
        self.global_dict.clear();
        self.template_dicts.clear();
        self.last_template_id = None;
    }

    /// Decodes an unsigned integer using stop-bit encoding.
    ///
    /// # Arguments
    /// * `data` - The input bytes
    /// * `offset` - Current position (will be updated)
    ///
    /// # Returns
    /// The decoded unsigned integer.
    ///
    /// # Errors
    /// Returns `FastError::UnexpectedEof` if data is incomplete.
    pub fn decode_uint(data: &[u8], offset: &mut usize) -> Result<u64, FastError> {
        let mut result: u64 = 0;

        loop {
            if *offset >= data.len() {
                return Err(FastError::UnexpectedEof);
            }

            let byte = data[*offset];
            *offset += 1;

            // Check for overflow
            if result > (u64::MAX >> 7) {
                return Err(FastError::IntegerOverflow);
            }

            result = (result << 7) | (byte & 0x7F) as u64;

            // Check stop bit
            if byte & 0x80 != 0 {
                break;
            }
        }

        Ok(result)
    }

    /// Decodes a signed integer using stop-bit encoding.
    ///
    /// # Arguments
    /// * `data` - The input bytes
    /// * `offset` - Current position (will be updated)
    ///
    /// # Returns
    /// The decoded signed integer.
    ///
    /// # Errors
    /// Returns `FastError::UnexpectedEof` if data is incomplete.
    pub fn decode_int(data: &[u8], offset: &mut usize) -> Result<i64, FastError> {
        if *offset >= data.len() {
            return Err(FastError::UnexpectedEof);
        }

        let first_byte = data[*offset];
        let negative = (first_byte & 0x40) != 0;

        let mut result: i64 = if negative { -1 } else { 0 };

        loop {
            if *offset >= data.len() {
                return Err(FastError::UnexpectedEof);
            }

            let byte = data[*offset];
            *offset += 1;

            result = (result << 7) | (byte & 0x7F) as i64;

            if byte & 0x80 != 0 {
                break;
            }
        }

        Ok(result)
    }

    /// Decodes an ASCII string using stop-bit encoding.
    ///
    /// # Arguments
    /// * `data` - The input bytes
    /// * `offset` - Current position (will be updated)
    ///
    /// # Returns
    /// The decoded string.
    ///
    /// # Errors
    /// Returns `FastError::UnexpectedEof` if data is incomplete.
    pub fn decode_ascii(data: &[u8], offset: &mut usize) -> Result<String, FastError> {
        let mut result = Vec::new();

        loop {
            if *offset >= data.len() {
                return Err(FastError::UnexpectedEof);
            }

            let byte = data[*offset];
            *offset += 1;

            // Add character (without stop bit)
            result.push(byte & 0x7F);

            // Check stop bit
            if byte & 0x80 != 0 {
                break;
            }
        }

        String::from_utf8(result).map_err(|_| FastError::InvalidString)
    }

    /// Decodes a byte vector.
    ///
    /// # Arguments
    /// * `data` - The input bytes
    /// * `offset` - Current position (will be updated)
    ///
    /// # Returns
    /// The decoded bytes.
    ///
    /// # Errors
    /// Returns `FastError::UnexpectedEof` if data is incomplete.
    pub fn decode_bytes(data: &[u8], offset: &mut usize) -> Result<Vec<u8>, FastError> {
        let length = Self::decode_uint(data, offset)? as usize;

        if *offset + length > data.len() {
            return Err(FastError::UnexpectedEof);
        }

        let bytes = data[*offset..*offset + length].to_vec();
        *offset += length;

        Ok(bytes)
    }

    /// Decodes a presence map.
    ///
    /// # Arguments
    /// * `data` - The input bytes
    /// * `offset` - Current position (will be updated)
    ///
    /// # Returns
    /// The decoded presence map.
    ///
    /// # Errors
    /// Returns `FastError::UnexpectedEof` if data is incomplete.
    pub fn decode_pmap(data: &[u8], offset: &mut usize) -> Result<PresenceMap, FastError> {
        PresenceMap::decode(data, offset)
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

    /// Gets a value from a template dictionary.
    #[must_use]
    pub fn get_template(&self, template_id: u32, key: &str) -> Option<&DictionaryValue> {
        self.template_dicts
            .get(&template_id)
            .and_then(|dict| dict.get(key))
    }

    /// Sets a value in a template dictionary.
    pub fn set_template(
        &mut self,
        template_id: u32,
        key: impl Into<String>,
        value: DictionaryValue,
    ) {
        self.template_dicts
            .entry(template_id)
            .or_default()
            .insert(key.into(), value);
    }

    /// Returns the last used template ID.
    #[must_use]
    pub const fn last_template_id(&self) -> Option<u32> {
        self.last_template_id
    }

    /// Sets the last used template ID.
    pub fn set_last_template_id(&mut self, id: u32) {
        self.last_template_id = Some(id);
    }
}

impl Default for FastDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_uint_single_byte() {
        let data = [0x81]; // 1 with stop bit
        let mut offset = 0;
        let result = FastDecoder::decode_uint(&data, &mut offset).unwrap();
        assert_eq!(result, 1);
        assert_eq!(offset, 1);
    }

    #[test]
    fn test_decode_uint_multi_byte() {
        let data = [0x00, 0x81]; // 1 in two bytes
        let mut offset = 0;
        let result = FastDecoder::decode_uint(&data, &mut offset).unwrap();
        assert_eq!(result, 1);
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_decode_uint_larger() {
        // 942 = 0x3AE = 0b11_1010_1110
        // In stop-bit encoding: 0x07 (0b0000111), 0xAE | 0x80 = 0x2E | 0x80 = 0xAE
        // Actually: 942 = 7 * 128 + 46 = 896 + 46
        // First byte: 7 (0x07), second byte: 46 | 0x80 = 0xAE
        let data = [0x07, 0xAE]; // 942 in stop-bit encoding
        let mut offset = 0;
        let result = FastDecoder::decode_uint(&data, &mut offset).unwrap();
        assert_eq!(result, 942);
    }

    #[test]
    fn test_decode_int_positive() {
        let data = [0x81]; // 1
        let mut offset = 0;
        let result = FastDecoder::decode_int(&data, &mut offset).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_decode_int_negative() {
        let data = [0xFF]; // -1
        let mut offset = 0;
        let result = FastDecoder::decode_int(&data, &mut offset).unwrap();
        assert_eq!(result, -1);
    }

    #[test]
    fn test_decode_ascii() {
        let data = [b'H', b'i', b'!' | 0x80]; // "Hi!"
        let mut offset = 0;
        let result = FastDecoder::decode_ascii(&data, &mut offset).unwrap();
        assert_eq!(result, "Hi!");
    }

    #[test]
    fn test_decoder_dictionary() {
        let mut decoder = FastDecoder::new();

        decoder.set_global("test", DictionaryValue::Int(42));
        assert_eq!(decoder.get_global("test").unwrap().as_i64(), Some(42));

        decoder.set_template(1, "field", DictionaryValue::UInt(100));
        assert_eq!(
            decoder.get_template(1, "field").unwrap().as_u64(),
            Some(100)
        );
    }
}
