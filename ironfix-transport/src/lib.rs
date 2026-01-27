/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! # IronFix Transport
//!
//! Network transport layer for the IronFix FIX protocol engine.
//!
//! This crate provides:
//! - **TCP transport**: Connector and acceptor for TCP connections
//! - **Codec**: Tokio codec for FIX message framing
//! - **TLS support**: Optional TLS encryption via rustls

pub mod codec;

pub use codec::{CodecError, FixCodec};
