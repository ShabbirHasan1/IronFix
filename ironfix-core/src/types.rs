/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! Core types for FIX protocol operations.
//!
//! This module provides fundamental types used throughout the IronFix engine:
//! - [`SeqNum`]: Sequence number wrapper with atomic operations
//! - [`Timestamp`]: FIX-formatted timestamp with nanosecond precision
//! - [`CompId`]: Component identifier (SenderCompID, TargetCompID)
//! - [`Side`]: Order side enumeration

use arrayvec::ArrayString;
use chrono::{DateTime, Utc};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Maximum length for CompID strings in bytes.
pub const COMP_ID_MAX_LEN: usize = 32;

/// FIX message sequence number.
///
/// Sequence numbers are unsigned 64-bit integers that identify messages
/// within a FIX session. They start at 1 and increment for each message sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SeqNum(u64);

impl SeqNum {
    /// Creates a new sequence number.
    ///
    /// # Arguments
    /// * `value` - The sequence number value (should be >= 1 for valid FIX messages)
    #[inline]
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw sequence number value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Returns the next sequence number.
    #[inline]
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// Checks if this sequence number is valid (>= 1).
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 >= 1
    }
}

impl Default for SeqNum {
    fn default() -> Self {
        Self(1)
    }
}

impl From<u64> for SeqNum {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<SeqNum> for u64 {
    fn from(seq: SeqNum) -> Self {
        seq.0
    }
}

impl fmt::Display for SeqNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// FIX protocol timestamp with nanosecond precision.
///
/// Timestamps in FIX are formatted as `YYYYMMDD-HH:MM:SS.sss` (milliseconds)
/// or `YYYYMMDD-HH:MM:SS.ssssss` (microseconds) or `YYYYMMDD-HH:MM:SS.sssssssss` (nanoseconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp {
    /// Nanoseconds since Unix epoch (1970-01-01 00:00:00 UTC).
    nanos_since_epoch: u64,
}

impl Timestamp {
    /// Creates a timestamp from nanoseconds since Unix epoch.
    ///
    /// # Arguments
    /// * `nanos` - Nanoseconds since 1970-01-01 00:00:00 UTC
    #[inline]
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self {
            nanos_since_epoch: nanos,
        }
    }

    /// Creates a timestamp from milliseconds since Unix epoch.
    ///
    /// # Arguments
    /// * `millis` - Milliseconds since 1970-01-01 00:00:00 UTC
    #[inline]
    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        Self {
            nanos_since_epoch: millis * 1_000_000,
        }
    }

    /// Returns the current UTC timestamp.
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        let dt = Utc::now();
        Self {
            nanos_since_epoch: dt.timestamp_nanos_opt().unwrap_or(0) as u64,
        }
    }

    /// Returns nanoseconds since Unix epoch.
    #[inline]
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.nanos_since_epoch
    }

    /// Returns milliseconds since Unix epoch.
    #[inline]
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.nanos_since_epoch / 1_000_000
    }

    /// Returns microseconds since Unix epoch.
    #[inline]
    #[must_use]
    pub const fn as_micros(self) -> u64 {
        self.nanos_since_epoch / 1_000
    }

    /// Converts to a chrono `DateTime<Utc>`.
    #[must_use]
    pub fn to_datetime(self) -> DateTime<Utc> {
        DateTime::from_timestamp_nanos(self.nanos_since_epoch as i64)
    }

    /// Formats the timestamp in FIX format with millisecond precision.
    ///
    /// Format: `YYYYMMDD-HH:MM:SS.sss`
    #[must_use]
    pub fn format_millis(self) -> ArrayString<21> {
        let dt = self.to_datetime();
        let mut buf = ArrayString::new();
        let _ = std::fmt::write(
            &mut buf,
            format_args!("{}", dt.format("%Y%m%d-%H:%M:%S%.3f")),
        );
        buf
    }

    /// Formats the timestamp in FIX format with microsecond precision.
    ///
    /// Format: `YYYYMMDD-HH:MM:SS.ssssss`
    #[must_use]
    pub fn format_micros(self) -> ArrayString<24> {
        let dt = self.to_datetime();
        let mut buf = ArrayString::new();
        let _ = std::fmt::write(
            &mut buf,
            format_args!("{}", dt.format("%Y%m%d-%H:%M:%S%.6f")),
        );
        buf
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self {
            nanos_since_epoch: dt.timestamp_nanos_opt().unwrap_or(0) as u64,
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_millis())
    }
}

/// Component identifier for FIX sessions.
///
/// Used for SenderCompID (tag 49), TargetCompID (tag 56), and related fields.
/// Maximum length is 32 characters as per FIX specification.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct CompId(ArrayString<COMP_ID_MAX_LEN>);

impl CompId {
    /// Creates a new CompId from a string slice.
    ///
    /// # Arguments
    /// * `s` - The component identifier string
    ///
    /// # Returns
    /// `Some(CompId)` if the string fits within the maximum length, `None` otherwise.
    #[must_use]
    pub fn new(s: &str) -> Option<Self> {
        ArrayString::from(s).ok().map(Self)
    }

    /// Returns the CompId as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the length of the CompId in bytes.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the CompId is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<str> for CompId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for CompId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for CompId {
    type Err = arrayvec::CapacityError<()>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ArrayString::try_from(s)
            .map(Self)
            .map_err(|_| arrayvec::CapacityError::new(()))
    }
}

/// Order side enumeration (tag 54).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, FromPrimitive, ToPrimitive,
)]
#[repr(u8)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    /// Buy order.
    Buy = b'1',
    /// Sell order.
    Sell = b'2',
    /// Buy minus (sell short exempt).
    BuyMinus = b'3',
    /// Sell plus (buy to cover).
    SellPlus = b'4',
    /// Sell short.
    SellShort = b'5',
    /// Sell short exempt.
    SellShortExempt = b'6',
    /// Undisclosed.
    Undisclosed = b'7',
    /// Cross (both sides).
    Cross = b'8',
    /// Cross short.
    CrossShort = b'9',
    /// Cross short exempt.
    CrossShortExempt = b'A',
    /// As defined (for multileg).
    AsDefined = b'B',
    /// Opposite (for multileg).
    Opposite = b'C',
    /// Subscribe.
    Subscribe = b'D',
    /// Redeem.
    Redeem = b'E',
    /// Lend (for securities lending).
    Lend = b'F',
    /// Borrow (for securities lending).
    Borrow = b'G',
}

impl Side {
    /// Creates a Side from a single character.
    ///
    /// # Arguments
    /// * `c` - The character representing the side
    ///
    /// # Returns
    /// `Some(Side)` if the character is valid, `None` otherwise.
    #[must_use]
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            '1' => Some(Self::Buy),
            '2' => Some(Self::Sell),
            '3' => Some(Self::BuyMinus),
            '4' => Some(Self::SellPlus),
            '5' => Some(Self::SellShort),
            '6' => Some(Self::SellShortExempt),
            '7' => Some(Self::Undisclosed),
            '8' => Some(Self::Cross),
            '9' => Some(Self::CrossShort),
            'A' => Some(Self::CrossShortExempt),
            'B' => Some(Self::AsDefined),
            'C' => Some(Self::Opposite),
            'D' => Some(Self::Subscribe),
            'E' => Some(Self::Redeem),
            'F' => Some(Self::Lend),
            'G' => Some(Self::Borrow),
            _ => None,
        }
    }

    /// Returns the character representation of this side.
    #[must_use]
    pub const fn as_char(self) -> char {
        self as u8 as char
    }

    /// Returns true if this is a buy-side order.
    #[must_use]
    pub const fn is_buy(self) -> bool {
        matches!(self, Self::Buy | Self::BuyMinus)
    }

    /// Returns true if this is a sell-side order.
    #[must_use]
    pub const fn is_sell(self) -> bool {
        matches!(
            self,
            Self::Sell | Self::SellPlus | Self::SellShort | Self::SellShortExempt
        )
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl TryFrom<u8> for Side {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_char(value as char).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq_num_operations() {
        let seq = SeqNum::new(5);
        assert_eq!(seq.value(), 5);
        assert_eq!(seq.next().value(), 6);
        assert!(seq.is_valid());
        assert!(!SeqNum::new(0).is_valid());
    }

    #[test]
    fn test_seq_num_default() {
        let seq = SeqNum::default();
        assert_eq!(seq.value(), 1);
    }

    #[test]
    fn test_timestamp_conversions() {
        let ts = Timestamp::from_millis(1000);
        assert_eq!(ts.as_millis(), 1000);
        assert_eq!(ts.as_micros(), 1_000_000);
        assert_eq!(ts.as_nanos(), 1_000_000_000);
    }

    #[test]
    fn test_timestamp_format() {
        let ts = Timestamp::from_millis(0);
        let formatted = ts.format_millis();
        assert!(formatted.starts_with("19700101-00:00:00"));
    }

    #[test]
    fn test_comp_id() {
        let id = CompId::new("SENDER").unwrap();
        assert_eq!(id.as_str(), "SENDER");
        assert_eq!(id.len(), 6);
        assert!(!id.is_empty());
    }

    #[test]
    fn test_comp_id_too_long() {
        let long_str = "A".repeat(COMP_ID_MAX_LEN + 1);
        assert!(CompId::new(&long_str).is_none());
    }

    #[test]
    fn test_side_from_char() {
        assert_eq!(Side::from_char('1'), Some(Side::Buy));
        assert_eq!(Side::from_char('2'), Some(Side::Sell));
        assert_eq!(Side::from_char('X'), None);
    }

    #[test]
    fn test_side_is_buy_sell() {
        assert!(Side::Buy.is_buy());
        assert!(!Side::Buy.is_sell());
        assert!(Side::Sell.is_sell());
        assert!(!Side::Sell.is_buy());
    }

    #[test]
    fn test_side_display() {
        assert_eq!(Side::Buy.to_string(), "1");
        assert_eq!(Side::Sell.to_string(), "2");
    }
}
