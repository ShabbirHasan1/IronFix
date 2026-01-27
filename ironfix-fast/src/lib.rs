/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! # IronFix FAST
//!
//! FAST (FIX Adapted for Streaming) protocol encoding and decoding for the IronFix engine.
//!
//! FAST is a binary encoding protocol used for high-performance market data feeds.
//! It uses techniques like stop-bit encoding, presence maps, and field operators
//! to achieve high compression ratios.
//!
//! ## Features
//!
//! - **Stop-bit encoding**: Efficient integer and string encoding
//! - **Presence maps**: Track which optional fields are present
//! - **Field operators**: Copy, Delta, Increment, Tail, etc.
//! - **Template support**: Message structure definitions

pub mod decoder;
pub mod encoder;
pub mod error;
pub mod operators;
pub mod pmap;

pub use decoder::FastDecoder;
pub use encoder::FastEncoder;
pub use error::FastError;
pub use pmap::PresenceMap;
