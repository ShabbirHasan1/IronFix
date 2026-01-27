/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! # IronFix Engine
//!
//! High-level FIX engine facade for the IronFix protocol implementation.
//!
//! This crate provides:
//! - **Initiator**: Client-side FIX engine for connecting to counterparties
//! - **Acceptor**: Server-side FIX engine for accepting connections
//! - **Application trait**: Callback interface for handling FIX messages
//! - **Builder API**: Fluent configuration for engine setup

pub mod application;
pub mod builder;

pub use application::Application;
pub use builder::EngineBuilder;
