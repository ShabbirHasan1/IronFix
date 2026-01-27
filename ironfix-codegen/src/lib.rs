/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! # IronFix Codegen
//!
//! Build-time code generation for the IronFix FIX protocol engine.
//!
//! This crate generates Rust source code from FIX dictionary definitions,
//! providing type-safe field constants and message structs.
//!
//! ## Usage
//!
//! Typically used in a `build.rs` script to generate code at compile time.

pub mod generator;

pub use generator::{CodeGenerator, GeneratorConfig};
