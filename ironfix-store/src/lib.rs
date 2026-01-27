/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! # IronFix Store
//!
//! Message persistence and storage for the IronFix FIX protocol engine.
//!
//! This crate provides:
//! - **MessageStore trait**: Abstract interface for message storage
//! - **MemoryStore**: In-memory message store for testing and simple use cases
//! - **FileStore**: File-based persistent message store

pub mod memory;
pub mod traits;

pub use memory::MemoryStore;
pub use traits::MessageStore;
