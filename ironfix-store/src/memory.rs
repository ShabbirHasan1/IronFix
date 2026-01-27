/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! In-memory message store implementation.
//!
//! This module provides a simple in-memory message store suitable for
//! testing and applications that don't require persistence.

use crate::traits::MessageStore;
use async_trait::async_trait;
use bytes::Bytes;
use ironfix_core::error::StoreError;
use ironfix_core::message::{MsgType, OwnedMessage};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

/// In-memory message store.
///
/// Stores messages in a `BTreeMap` for efficient range queries.
/// Not persistent - all data is lost when the process exits.
#[derive(Debug)]
pub struct MemoryStore {
    /// Stored messages indexed by sequence number.
    messages: RwLock<BTreeMap<u64, Bytes>>,
    /// Next sender sequence number.
    next_sender_seq: AtomicU64,
    /// Next expected target sequence number.
    next_target_seq: AtomicU64,
    /// Store creation time.
    creation_time: SystemTime,
}

impl MemoryStore {
    /// Creates a new empty memory store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            messages: RwLock::new(BTreeMap::new()),
            next_sender_seq: AtomicU64::new(1),
            next_target_seq: AtomicU64::new(1),
            creation_time: SystemTime::now(),
        }
    }

    /// Creates a new memory store with initial sequence numbers.
    ///
    /// # Arguments
    /// * `sender_seq` - Initial sender sequence number
    /// * `target_seq` - Initial target sequence number
    #[must_use]
    pub fn with_initial_seqs(sender_seq: u64, target_seq: u64) -> Self {
        Self {
            messages: RwLock::new(BTreeMap::new()),
            next_sender_seq: AtomicU64::new(sender_seq),
            next_target_seq: AtomicU64::new(target_seq),
            creation_time: SystemTime::now(),
        }
    }

    /// Returns the number of stored messages.
    #[must_use]
    pub fn message_count(&self) -> usize {
        self.messages.read().len()
    }

    /// Checks if a message with the given sequence number exists.
    #[must_use]
    pub fn contains(&self, seq_num: u64) -> bool {
        self.messages.read().contains_key(&seq_num)
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageStore for MemoryStore {
    async fn store(&self, seq_num: u64, message: &[u8]) -> Result<(), StoreError> {
        let mut messages = self.messages.write();
        messages.insert(seq_num, Bytes::copy_from_slice(message));
        Ok(())
    }

    async fn get_range(&self, begin: u64, end: u64) -> Result<Vec<OwnedMessage>, StoreError> {
        let messages = self.messages.read();
        let end = if end == 0 { u64::MAX } else { end };

        let result: Vec<OwnedMessage> = messages
            .range(begin..=end)
            .map(|(_, bytes)| OwnedMessage::new(bytes.clone(), MsgType::default(), vec![]))
            .collect();

        if result.is_empty() && begin <= end {
            return Err(StoreError::RangeNotAvailable {
                range: begin..end + 1,
            });
        }

        Ok(result)
    }

    fn next_sender_seq(&self) -> u64 {
        self.next_sender_seq.load(Ordering::SeqCst)
    }

    fn next_target_seq(&self) -> u64 {
        self.next_target_seq.load(Ordering::SeqCst)
    }

    fn set_next_sender_seq(&self, seq: u64) {
        self.next_sender_seq.store(seq, Ordering::SeqCst);
    }

    fn set_next_target_seq(&self, seq: u64) {
        self.next_target_seq.store(seq, Ordering::SeqCst);
    }

    async fn reset(&self) -> Result<(), StoreError> {
        let mut messages = self.messages.write();
        messages.clear();
        self.next_sender_seq.store(1, Ordering::SeqCst);
        self.next_target_seq.store(1, Ordering::SeqCst);
        Ok(())
    }

    fn creation_time(&self) -> SystemTime {
        self.creation_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_store_new() {
        let store = MemoryStore::new();
        assert_eq!(store.next_sender_seq(), 1);
        assert_eq!(store.next_target_seq(), 1);
        assert_eq!(store.message_count(), 0);
    }

    #[tokio::test]
    async fn test_memory_store_store_and_retrieve() {
        let store = MemoryStore::new();

        store.store(1, b"message1").await.unwrap();
        store.store(2, b"message2").await.unwrap();
        store.store(3, b"message3").await.unwrap();

        assert_eq!(store.message_count(), 3);
        assert!(store.contains(1));
        assert!(store.contains(2));
        assert!(store.contains(3));
        assert!(!store.contains(4));
    }

    #[tokio::test]
    async fn test_memory_store_get_range() {
        let store = MemoryStore::new();

        store.store(1, b"msg1").await.unwrap();
        store.store(2, b"msg2").await.unwrap();
        store.store(3, b"msg3").await.unwrap();
        store.store(5, b"msg5").await.unwrap();

        let range = store.get_range(1, 3).await.unwrap();
        assert_eq!(range.len(), 3);

        let range = store.get_range(2, 5).await.unwrap();
        assert_eq!(range.len(), 3);
    }

    #[tokio::test]
    async fn test_memory_store_sequence_numbers() {
        let store = MemoryStore::new();

        store.set_next_sender_seq(10);
        store.set_next_target_seq(20);

        assert_eq!(store.next_sender_seq(), 10);
        assert_eq!(store.next_target_seq(), 20);
    }

    #[tokio::test]
    async fn test_memory_store_reset() {
        let store = MemoryStore::new();

        store.store(1, b"msg1").await.unwrap();
        store.set_next_sender_seq(10);
        store.set_next_target_seq(20);

        store.reset().await.unwrap();

        assert_eq!(store.message_count(), 0);
        assert_eq!(store.next_sender_seq(), 1);
        assert_eq!(store.next_target_seq(), 1);
    }
}
