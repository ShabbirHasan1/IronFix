/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! # IronFix Tag-Value
//!
//! Zero-copy FIX tag=value encoding and decoding for the IronFix engine.
//!
//! This crate provides high-performance parsing and serialization of FIX messages
//! using the standard tag=value format with SOH (0x01) delimiters.
//!
//! ## Features
//!
//! - **Zero-copy parsing**: Field values reference the original buffer
//! - **SIMD-accelerated**: Uses `memchr` for fast delimiter search
//! - **Checksum calculation**: Optimized checksum computation

pub mod checksum;
pub mod decoder;
pub mod encoder;

pub use checksum::calculate_checksum;
pub use decoder::Decoder;
pub use encoder::Encoder;
pub use ironfix_core::message::RawMessage;
