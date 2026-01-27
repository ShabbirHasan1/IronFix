/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! Sequence number management.
//!
//! This module provides atomic sequence number management for FIX sessions.

use ironfix_core::types::SeqNum;
use std::sync::atomic::{AtomicU64, Ordering};

/// Manages sequence numbers for a FIX session.
///
/// Uses atomic operations for thread-safe access without locks.
#[derive(Debug)]
pub struct SequenceManager {
    /// Next outgoing sequence number.
    next_sender_seq: AtomicU64,
    /// Next expected incoming sequence number.
    next_target_seq: AtomicU64,
}

impl SequenceManager {
    /// Creates a new sequence manager with sequence numbers starting at 1.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_sender_seq: AtomicU64::new(1),
            next_target_seq: AtomicU64::new(1),
        }
    }

    /// Creates a new sequence manager with specified starting values.
    ///
    /// # Arguments
    /// * `sender_seq` - Initial sender sequence number
    /// * `target_seq` - Initial target sequence number
    #[must_use]
    pub fn with_initial(sender_seq: u64, target_seq: u64) -> Self {
        Self {
            next_sender_seq: AtomicU64::new(sender_seq),
            next_target_seq: AtomicU64::new(target_seq),
        }
    }

    /// Returns the next sender sequence number without incrementing.
    #[inline]
    #[must_use]
    pub fn next_sender_seq(&self) -> SeqNum {
        SeqNum::new(self.next_sender_seq.load(Ordering::SeqCst))
    }

    /// Returns the next target sequence number without incrementing.
    #[inline]
    #[must_use]
    pub fn next_target_seq(&self) -> SeqNum {
        SeqNum::new(self.next_target_seq.load(Ordering::SeqCst))
    }

    /// Allocates and returns the next sender sequence number.
    ///
    /// This atomically increments the sequence number and returns the
    /// value before the increment.
    #[inline]
    pub fn allocate_sender_seq(&self) -> SeqNum {
        SeqNum::new(self.next_sender_seq.fetch_add(1, Ordering::SeqCst))
    }

    /// Increments the target sequence number.
    ///
    /// Call this after successfully processing an incoming message.
    #[inline]
    pub fn increment_target_seq(&self) {
        self.next_target_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Sets the next sender sequence number.
    ///
    /// # Arguments
    /// * `seq` - The new sequence number
    #[inline]
    pub fn set_sender_seq(&self, seq: u64) {
        self.next_sender_seq.store(seq, Ordering::SeqCst);
    }

    /// Sets the next target sequence number.
    ///
    /// # Arguments
    /// * `seq` - The new sequence number
    #[inline]
    pub fn set_target_seq(&self, seq: u64) {
        self.next_target_seq.store(seq, Ordering::SeqCst);
    }

    /// Resets both sequence numbers to 1.
    #[inline]
    pub fn reset(&self) {
        self.next_sender_seq.store(1, Ordering::SeqCst);
        self.next_target_seq.store(1, Ordering::SeqCst);
    }

    /// Validates an incoming sequence number.
    ///
    /// # Arguments
    /// * `received` - The received sequence number
    ///
    /// # Returns
    /// - `Ok(())` if the sequence number matches expected
    /// - `Err(SequenceResult::TooLow)` if it's a possible duplicate
    /// - `Err(SequenceResult::Gap)` if there's a gap
    #[must_use]
    pub fn validate_incoming(&self, received: u64) -> SequenceResult {
        let expected = self.next_target_seq.load(Ordering::SeqCst);

        if received == expected {
            SequenceResult::Ok
        } else if received < expected {
            SequenceResult::TooLow { expected, received }
        } else {
            SequenceResult::Gap { expected, received }
        }
    }
}

impl Default for SequenceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of sequence number validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceResult {
    /// Sequence number is as expected.
    Ok,
    /// Sequence number is lower than expected (possible duplicate).
    TooLow {
        /// Expected sequence number.
        expected: u64,
        /// Received sequence number.
        received: u64,
    },
    /// Sequence number is higher than expected (gap detected).
    Gap {
        /// Expected sequence number.
        expected: u64,
        /// Received sequence number.
        received: u64,
    },
}

impl SequenceResult {
    /// Returns true if the sequence is valid.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Returns true if there's a gap.
    #[must_use]
    pub const fn is_gap(&self) -> bool {
        matches!(self, Self::Gap { .. })
    }

    /// Returns true if the sequence is too low.
    #[must_use]
    pub const fn is_too_low(&self) -> bool {
        matches!(self, Self::TooLow { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_manager_new() {
        let mgr = SequenceManager::new();
        assert_eq!(mgr.next_sender_seq().value(), 1);
        assert_eq!(mgr.next_target_seq().value(), 1);
    }

    #[test]
    fn test_allocate_sender_seq() {
        let mgr = SequenceManager::new();

        let seq1 = mgr.allocate_sender_seq();
        assert_eq!(seq1.value(), 1);
        assert_eq!(mgr.next_sender_seq().value(), 2);

        let seq2 = mgr.allocate_sender_seq();
        assert_eq!(seq2.value(), 2);
        assert_eq!(mgr.next_sender_seq().value(), 3);
    }

    #[test]
    fn test_increment_target_seq() {
        let mgr = SequenceManager::new();

        mgr.increment_target_seq();
        assert_eq!(mgr.next_target_seq().value(), 2);

        mgr.increment_target_seq();
        assert_eq!(mgr.next_target_seq().value(), 3);
    }

    #[test]
    fn test_validate_incoming() {
        let mgr = SequenceManager::new();

        assert!(mgr.validate_incoming(1).is_ok());

        mgr.set_target_seq(5);
        assert!(mgr.validate_incoming(4).is_too_low());
        assert!(mgr.validate_incoming(5).is_ok());
        assert!(mgr.validate_incoming(10).is_gap());
    }

    #[test]
    fn test_reset() {
        let mgr = SequenceManager::with_initial(100, 200);
        assert_eq!(mgr.next_sender_seq().value(), 100);
        assert_eq!(mgr.next_target_seq().value(), 200);

        mgr.reset();
        assert_eq!(mgr.next_sender_seq().value(), 1);
        assert_eq!(mgr.next_target_seq().value(), 1);
    }
}
