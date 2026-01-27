/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! # IronFix Dictionary
//!
//! FIX specification parsing and dictionary management for the IronFix engine.
//!
//! This crate provides:
//! - **Schema definitions**: Field, message, and component definitions
//! - **Dictionary parsing**: QuickFIX XML format parser
//! - **Runtime validation**: Message validation against dictionary rules
//! - **Embedded dictionaries**: Pre-loaded FIX 4.0 through 5.0 SP2 specifications

pub mod schema;

pub use schema::{ComponentDef, Dictionary, FieldDef, FieldType, GroupDef, MessageDef, Version};
