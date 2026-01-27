/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! Message store trait definition.
//!
//! This module defines the abstract interface for message storage implementations.

use async_trait::async_trait;
use ironfix_core::error::StoreError;
use ironfix_core::message::OwnedMessage;

/// Abstract interface for FIX message storage.
///
/// Implementations of this trait provide persistence for outgoing messages
/// to support resend requests and session recovery.
#[async_trait]
pub trait MessageStore: Send + Sync {
    /// Stores an outgoing message for potential resend.
    ///
    /// # Arguments
    /// * `seq_num` - The message sequence number
    /// * `message` - The raw message bytes
    ///
    /// # Errors
    /// Returns `StoreError` if the message cannot be stored.
    async fn store(&self, seq_num: u64, message: &[u8]) -> Result<(), StoreError>;

    /// Retrieves messages for a resend request.
    ///
    /// # Arguments
    /// * `begin` - Begin sequence number (inclusive)
    /// * `end` - End sequence number (inclusive, or 0 for infinity)
    ///
    /// # Returns
    /// A vector of messages in the requested range.
    ///
    /// # Errors
    /// Returns `StoreError` if messages cannot be retrieved.
    async fn get_range(&self, begin: u64, end: u64) -> Result<Vec<OwnedMessage>, StoreError>;

    /// Returns the next sender sequence number.
    fn next_sender_seq(&self) -> u64;

    /// Returns the next expected target sequence number.
    fn next_target_seq(&self) -> u64;

    /// Sets the next sender sequence number.
    ///
    /// # Arguments
    /// * `seq` - The new sequence number
    fn set_next_sender_seq(&self, seq: u64);

    /// Sets the next expected target sequence number.
    ///
    /// # Arguments
    /// * `seq` - The new sequence number
    fn set_next_target_seq(&self, seq: u64);

    /// Resets the store, clearing all messages and resetting sequence numbers.
    ///
    /// # Errors
    /// Returns `StoreError` if the reset fails.
    async fn reset(&self) -> Result<(), StoreError>;

    /// Returns the creation time of the store/session.
    fn creation_time(&self) -> std::time::SystemTime;

    /// Refreshes the store from persistent storage.
    ///
    /// # Errors
    /// Returns `StoreError` if the refresh fails.
    async fn refresh(&self) -> Result<(), StoreError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockStore;

    #[async_trait]
    impl MessageStore for MockStore {
        async fn store(&self, _seq_num: u64, _message: &[u8]) -> Result<(), StoreError> {
            Ok(())
        }

        async fn get_range(&self, _begin: u64, _end: u64) -> Result<Vec<OwnedMessage>, StoreError> {
            Ok(vec![])
        }

        fn next_sender_seq(&self) -> u64 {
            1
        }

        fn next_target_seq(&self) -> u64 {
            1
        }

        fn set_next_sender_seq(&self, _seq: u64) {}

        fn set_next_target_seq(&self, _seq: u64) {}

        async fn reset(&self) -> Result<(), StoreError> {
            Ok(())
        }

        fn creation_time(&self) -> std::time::SystemTime {
            std::time::SystemTime::now()
        }
    }

    #[tokio::test]
    async fn test_mock_store() {
        let store = MockStore;
        assert_eq!(store.next_sender_seq(), 1);
        assert_eq!(store.next_target_seq(), 1);
        assert!(store.store(1, b"test").await.is_ok());
        assert!(store.reset().await.is_ok());
    }
}
