/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! # IronFix
//!
//! A high-performance FIX/FAST protocol engine for Rust.
//!
//! IronFix provides a complete implementation of the FIX protocol with support for
//! all versions from FIX 4.0 through FIX 5.0 SP2, as well as FAST-encoded market data.
//!
//! ## Features
//!
//! - **Zero-copy parsing**: Field values reference the original buffer
//! - **SIMD-accelerated**: Uses `memchr` for fast delimiter search
//! - **Type-safe**: Compile-time checked session states and message types
//! - **Async support**: Built on Tokio for high-performance networking
//! - **Flexible**: Supports both sync and async operation modes
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use ironfix::prelude::*;
//!
//! // Create an engine with your application handler
//! let engine = EngineBuilder::new()
//!     .with_application(MyApplication)
//!     .add_session(SessionConfig::new(
//!         CompId::new("SENDER").unwrap(),
//!         CompId::new("TARGET").unwrap(),
//!         "FIX.4.4",
//!     ))
//!     .build();
//! ```
//!
//! ## Crate Organization
//!
//! - [`core`]: Fundamental types, traits, and error definitions
//! - [`dictionary`]: FIX specification parsing and dictionary management
//! - [`tagvalue`]: Zero-copy tag=value encoding and decoding
//! - [`session`]: Session layer protocol implementation
//! - [`store`]: Message persistence and storage
//! - [`transport`]: Network transport layer
//! - [`fast`]: FAST protocol encoding and decoding
//! - [`engine`]: High-level engine facade

pub mod core {
    //! Core types, traits, and error definitions.
    pub use ironfix_core::*;
}

pub mod dictionary {
    //! FIX specification parsing and dictionary management.
    pub use ironfix_dictionary::*;
}

pub mod tagvalue {
    //! Zero-copy tag=value encoding and decoding.
    pub use ironfix_tagvalue::*;
}

pub mod session {
    //! Session layer protocol implementation.
    pub use ironfix_session::*;
}

pub mod store {
    //! Message persistence and storage.
    pub use ironfix_store::*;
}

pub mod transport {
    //! Network transport layer.
    pub use ironfix_transport::*;
}

pub mod fast {
    //! FAST protocol encoding and decoding.
    pub use ironfix_fast::*;
}

pub mod engine {
    //! High-level engine facade.
    pub use ironfix_engine::*;
}

/// Prelude module for convenient imports.
pub mod prelude {
    // Core types
    pub use ironfix_core::{
        CompId, DecodeError, EncodeError, FieldRef, FieldTag, FieldValue, FixError, FixField,
        FixMessage, MsgType, OwnedMessage, RawMessage, Result, SeqNum, SessionError, Side,
        StoreError, Timestamp,
    };

    // Dictionary
    pub use ironfix_dictionary::{Dictionary, FieldDef, FieldType, MessageDef, Version};

    // Tag-value encoding
    pub use ironfix_tagvalue::{Decoder, Encoder, calculate_checksum};

    // Session
    pub use ironfix_session::{
        Active, Connecting, Disconnected, HeartbeatManager, LogonSent, LogoutPending, Resending,
        SequenceManager, SessionConfig, SessionState,
    };

    // Store
    pub use ironfix_store::{MemoryStore, MessageStore};

    // Transport
    pub use ironfix_transport::{CodecError, FixCodec};

    // FAST
    pub use ironfix_fast::{FastDecoder, FastEncoder, FastError, PresenceMap};

    // Engine
    pub use ironfix_engine::{Application, EngineBuilder};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_prelude_imports() {
        // Verify that prelude imports work
        let _seq = SeqNum::new(1);
        let _ts = Timestamp::now();
        let _side = Side::Buy;
    }

    #[test]
    fn test_version() {
        let version = Version::Fix44;
        assert_eq!(version.begin_string(), "FIX.4.4");
    }
}
