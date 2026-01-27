/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! # IronFix Core
//!
//! Core types, traits, and error definitions for the IronFix FIX protocol engine.
//!
//! This crate provides the fundamental building blocks used across all IronFix crates:
//! - **Error types**: Unified error handling with `thiserror`
//! - **Field types**: `FieldTag`, `FieldValue`, and the `FixField` trait
//! - **Message types**: `RawMessage`, `OwnedMessage`, and the `FixMessage` trait
//! - **Core types**: `SeqNum`, `Timestamp`, `CompID`, `MsgType`
//!
//! ## Zero-Copy Design
//!
//! The core abstractions support both zero-copy borrowed views (for hot-path processing)
//! and owned representations (for storage and cross-thread transfer).

pub mod error;
pub mod field;
pub mod message;
pub mod types;

pub use error::{DecodeError, EncodeError, FixError, Result, SessionError, StoreError};
pub use field::{FieldRef, FieldTag, FieldValue, FixField};
pub use message::{FixMessage, MsgType, OwnedMessage, RawMessage};
pub use types::{CompId, SeqNum, Side, Timestamp};
