/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! FAST presence map handling.
//!
//! The presence map (PMAP) is a bitmap that indicates which optional fields
//! are present in a FAST message. It uses stop-bit encoding where the high
//! bit of each byte indicates whether more bytes follow.

use crate::error::FastError;

/// FAST presence map.
///
/// The presence map tracks which optional fields are present in a message.
/// Bits are consumed in order as fields are decoded.
#[derive(Debug, Clone)]
pub struct PresenceMap {
    /// The raw bits of the presence map.
    bits: Vec<bool>,
    /// Current bit position.
    position: usize,
}

impl PresenceMap {
    /// Creates an empty presence map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bits: Vec::new(),
            position: 0,
        }
    }

    /// Creates a presence map from raw bits.
    #[must_use]
    pub fn from_bits(bits: Vec<bool>) -> Self {
        Self { bits, position: 0 }
    }

    /// Decodes a presence map from a byte slice.
    ///
    /// # Arguments
    /// * `data` - The input bytes
    /// * `offset` - Current position in the data (will be updated)
    ///
    /// # Returns
    /// The decoded presence map.
    ///
    /// # Errors
    /// Returns `FastError::UnexpectedEof` if the data is incomplete.
    pub fn decode(data: &[u8], offset: &mut usize) -> Result<Self, FastError> {
        let mut bits = Vec::new();

        loop {
            if *offset >= data.len() {
                return Err(FastError::UnexpectedEof);
            }

            let byte = data[*offset];
            *offset += 1;

            // Extract 7 bits (excluding stop bit)
            for i in (0..7).rev() {
                bits.push((byte >> i) & 1 == 1);
            }

            // Check stop bit (high bit)
            if byte & 0x80 != 0 {
                break;
            }
        }

        Ok(Self { bits, position: 0 })
    }

    /// Returns the next bit from the presence map.
    ///
    /// # Returns
    /// `true` if the field is present, `false` otherwise.
    /// Returns `false` if the map is exhausted.
    #[inline]
    pub fn next_bit(&mut self) -> bool {
        if self.position < self.bits.len() {
            let bit = self.bits[self.position];
            self.position += 1;
            bit
        } else {
            false
        }
    }

    /// Returns the bit at the specified position without consuming it.
    ///
    /// # Arguments
    /// * `index` - The bit position (0-indexed)
    #[must_use]
    pub fn bit(&self, index: usize) -> bool {
        self.bits.get(index).copied().unwrap_or(false)
    }

    /// Returns the number of bits in the presence map.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Returns true if the presence map is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    /// Returns the current position in the presence map.
    #[must_use]
    pub fn position(&self) -> usize {
        self.position
    }

    /// Resets the position to the beginning.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Encodes the presence map to bytes.
    ///
    /// # Returns
    /// The encoded bytes with stop-bit encoding.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        if self.bits.is_empty() {
            return vec![0x80]; // Empty pmap with stop bit
        }

        let mut result = Vec::new();
        let mut bit_index = 0;

        while bit_index < self.bits.len() {
            let mut byte: u8 = 0;

            // Pack 7 bits into each byte
            for i in (0..7).rev() {
                if bit_index < self.bits.len() && self.bits[bit_index] {
                    byte |= 1 << i;
                }
                bit_index += 1;
            }

            // Set stop bit if this is the last byte
            if bit_index >= self.bits.len() {
                byte |= 0x80;
            }

            result.push(byte);
        }

        result
    }
}

impl Default for PresenceMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing presence maps.
#[derive(Debug, Default)]
pub struct PresenceMapBuilder {
    bits: Vec<bool>,
}

impl PresenceMapBuilder {
    /// Creates a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a bit to the presence map.
    #[must_use]
    pub fn bit(mut self, present: bool) -> Self {
        self.bits.push(present);
        self
    }

    /// Builds the presence map.
    #[must_use]
    pub fn build(self) -> PresenceMap {
        PresenceMap::from_bits(self.bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_map_decode_single_byte() {
        // 0b1100_0000: stop bit (bit 7) = 1, bits 6-0 = 100_0000
        // Extracted bits (from bit 6 to bit 0): 1, 0, 0, 0, 0, 0, 0
        let data = [0b1100_0000];
        let mut offset = 0;
        let pmap = PresenceMap::decode(&data, &mut offset).unwrap();

        assert_eq!(offset, 1);
        assert_eq!(pmap.len(), 7);
        assert!(pmap.bit(0)); // bit 6 of byte = 1
        assert!(!pmap.bit(1)); // bit 5 of byte = 0
        assert!(!pmap.bit(2)); // bit 4 of byte = 0
    }

    #[test]
    fn test_presence_map_decode_multi_byte() {
        let data = [0b0100_0000, 0b1000_0000]; // Two bytes
        let mut offset = 0;
        let pmap = PresenceMap::decode(&data, &mut offset).unwrap();

        assert_eq!(offset, 2);
        assert_eq!(pmap.len(), 14);
    }

    #[test]
    fn test_presence_map_next_bit() {
        let mut pmap = PresenceMap::from_bits(vec![true, false, true]);

        assert!(pmap.next_bit());
        assert!(!pmap.next_bit());
        assert!(pmap.next_bit());
        assert!(!pmap.next_bit()); // Exhausted
    }

    #[test]
    fn test_presence_map_encode() {
        let pmap = PresenceMap::from_bits(vec![true, true, false, false, false, false, false]);
        let encoded = pmap.encode();

        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 0b1110_0000);
    }

    #[test]
    fn test_presence_map_builder() {
        let pmap = PresenceMapBuilder::new()
            .bit(true)
            .bit(false)
            .bit(true)
            .build();

        assert_eq!(pmap.len(), 3);
        assert!(pmap.bit(0));
        assert!(!pmap.bit(1));
        assert!(pmap.bit(2));
    }
}
