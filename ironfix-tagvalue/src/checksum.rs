/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! FIX checksum calculation.
//!
//! The FIX checksum is the sum of all bytes in the message (excluding the
//! checksum field itself) modulo 256, formatted as a 3-digit zero-padded string.

/// Calculates the FIX checksum for the given data.
///
/// The checksum is the sum of all bytes modulo 256.
///
/// # Arguments
/// * `data` - The message bytes to checksum (excluding the 10=XXX| field)
///
/// # Returns
/// The checksum value as a u8 (0-255).
///
/// # Example
/// ```
/// use ironfix_tagvalue::calculate_checksum;
///
/// let data = b"8=FIX.4.4\x019=5\x0135=0\x01";
/// let checksum = calculate_checksum(data);
/// ```
#[inline]
#[must_use]
pub fn calculate_checksum(data: &[u8]) -> u8 {
    calculate_checksum_portable(data)
}

/// Portable checksum calculation without SIMD.
#[inline]
fn calculate_checksum_portable(data: &[u8]) -> u8 {
    let sum: u32 = data.iter().map(|&b| b as u32).sum();
    (sum % 256) as u8
}

/// Formats a checksum value as a 3-digit zero-padded string.
///
/// # Arguments
/// * `checksum` - The checksum value (0-255)
///
/// # Returns
/// A 3-character string representation (e.g., "042", "255").
#[inline]
#[must_use]
pub fn format_checksum(checksum: u8) -> [u8; 3] {
    let d0 = b'0' + (checksum / 100);
    let d1 = b'0' + ((checksum / 10) % 10);
    let d2 = b'0' + (checksum % 10);
    [d0, d1, d2]
}

/// Parses a 3-digit checksum string to a u8 value.
///
/// # Arguments
/// * `bytes` - The 3-byte checksum string
///
/// # Returns
/// `Some(checksum)` if valid, `None` otherwise.
#[inline]
#[must_use]
pub fn parse_checksum(bytes: &[u8]) -> Option<u8> {
    if bytes.len() != 3 {
        return None;
    }

    let d0 = bytes[0].checked_sub(b'0')?;
    let d1 = bytes[1].checked_sub(b'0')?;
    let d2 = bytes[2].checked_sub(b'0')?;

    if d0 > 9 || d1 > 9 || d2 > 9 {
        return None;
    }

    Some(d0 * 100 + d1 * 10 + d2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_checksum_empty() {
        assert_eq!(calculate_checksum(b""), 0);
    }

    #[test]
    fn test_calculate_checksum_simple() {
        let data = b"ABC";
        let expected = (b'A' as u32 + b'B' as u32 + b'C' as u32) % 256;
        assert_eq!(calculate_checksum(data), expected as u8);
    }

    #[test]
    fn test_calculate_checksum_overflow() {
        let data = vec![255u8; 1000];
        let expected = ((255u32 * 1000) % 256) as u8;
        assert_eq!(calculate_checksum(&data), expected);
    }

    #[test]
    fn test_format_checksum() {
        assert_eq!(format_checksum(0), *b"000");
        assert_eq!(format_checksum(42), *b"042");
        assert_eq!(format_checksum(100), *b"100");
        assert_eq!(format_checksum(255), *b"255");
    }

    #[test]
    fn test_parse_checksum() {
        assert_eq!(parse_checksum(b"000"), Some(0));
        assert_eq!(parse_checksum(b"042"), Some(42));
        assert_eq!(parse_checksum(b"100"), Some(100));
        assert_eq!(parse_checksum(b"255"), Some(255));
    }

    #[test]
    fn test_parse_checksum_invalid() {
        assert_eq!(parse_checksum(b""), None);
        assert_eq!(parse_checksum(b"00"), None);
        assert_eq!(parse_checksum(b"0000"), None);
        assert_eq!(parse_checksum(b"abc"), None);
        assert_eq!(parse_checksum(b"12X"), None);
    }

    #[test]
    fn test_roundtrip() {
        for i in 0..=255u8 {
            let formatted = format_checksum(i);
            let parsed = parse_checksum(&formatted);
            assert_eq!(parsed, Some(i));
        }
    }
}
