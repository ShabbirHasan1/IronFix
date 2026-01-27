/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! Code generator for FIX dictionaries.
//!
//! Generates Rust source code from FIX dictionary definitions.

use ironfix_dictionary::schema::{Dictionary, FieldDef, FieldType, MessageDef};
use std::fmt::Write;

/// Configuration for code generation.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Whether to generate field constants.
    pub generate_fields: bool,
    /// Whether to generate message structs.
    pub generate_messages: bool,
    /// Whether to generate component traits.
    pub generate_components: bool,
    /// Module visibility (e.g., "pub", "pub(crate)").
    pub visibility: String,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            generate_fields: true,
            generate_messages: true,
            generate_components: true,
            visibility: "pub".to_string(),
        }
    }
}

/// Code generator for FIX dictionaries.
#[derive(Debug)]
pub struct CodeGenerator {
    config: GeneratorConfig,
}

impl CodeGenerator {
    /// Creates a new code generator with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GeneratorConfig::default(),
        }
    }

    /// Creates a new code generator with the specified configuration.
    #[must_use]
    pub fn with_config(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Generates Rust source code from a dictionary.
    ///
    /// # Arguments
    /// * `dict` - The FIX dictionary to generate code from
    ///
    /// # Returns
    /// The generated Rust source code as a string.
    #[must_use]
    pub fn generate(&self, dict: &Dictionary) -> String {
        let mut code = String::new();

        // File header
        writeln!(code, "//! Generated FIX {} definitions.", dict.version).unwrap();
        writeln!(code, "//!").unwrap();
        writeln!(
            code,
            "//! This file was automatically generated. Do not edit."
        )
        .unwrap();
        writeln!(code).unwrap();

        if self.config.generate_fields {
            self.generate_fields_module(&mut code, dict);
        }

        if self.config.generate_messages {
            self.generate_messages_module(&mut code, dict);
        }

        code
    }

    /// Generates the fields module with tag constants.
    fn generate_fields_module(&self, code: &mut String, dict: &Dictionary) {
        writeln!(code, "/// Field tag constants.").unwrap();
        writeln!(code, "{} mod fields {{", self.config.visibility).unwrap();

        let mut fields: Vec<_> = dict.fields().collect();
        fields.sort_by_key(|f| f.tag);

        for field in fields {
            self.generate_field_constant(code, field);
        }

        writeln!(code, "}}").unwrap();
        writeln!(code).unwrap();
    }

    /// Generates a single field constant.
    fn generate_field_constant(&self, code: &mut String, field: &FieldDef) {
        let const_name = to_screaming_snake_case(&field.name);

        if let Some(ref desc) = field.description {
            writeln!(code, "    /// {}", desc).unwrap();
        }
        writeln!(code, "    pub const {}: u32 = {};", const_name, field.tag).unwrap();
    }

    /// Generates the messages module with message structs.
    fn generate_messages_module(&self, code: &mut String, dict: &Dictionary) {
        writeln!(code, "/// Message type definitions.").unwrap();
        writeln!(code, "{} mod messages {{", self.config.visibility).unwrap();
        writeln!(code, "    use super::fields;").unwrap();
        writeln!(code).unwrap();

        let mut messages: Vec<_> = dict.messages().collect();
        messages.sort_by(|a, b| a.msg_type.cmp(&b.msg_type));

        for msg in messages {
            self.generate_message_struct(code, msg, dict);
        }

        writeln!(code, "}}").unwrap();
    }

    /// Generates a message struct.
    fn generate_message_struct(&self, code: &mut String, msg: &MessageDef, dict: &Dictionary) {
        let struct_name = to_pascal_case(&msg.name);

        writeln!(
            code,
            "    /// {} message (MsgType={}).",
            msg.name, msg.msg_type
        )
        .unwrap();
        writeln!(code, "    #[derive(Debug, Clone)]").unwrap();
        writeln!(code, "    pub struct {} {{", struct_name).unwrap();

        for field_ref in &msg.fields {
            if let Some(field_def) = dict.get_field(field_ref.tag) {
                let field_name = to_snake_case(&field_ref.name);
                let rust_type = field_type_to_rust(&field_def.field_type);

                if field_ref.required {
                    writeln!(code, "        pub {}: {},", field_name, rust_type).unwrap();
                } else {
                    writeln!(code, "        pub {}: Option<{}>,", field_name, rust_type).unwrap();
                }
            }
        }

        writeln!(code, "    }}").unwrap();
        writeln!(code).unwrap();
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts a string to SCREAMING_SNAKE_CASE.
fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_lower = false;

    for c in s.chars() {
        if c.is_uppercase() && prev_lower {
            result.push('_');
        }
        result.push(c.to_ascii_uppercase());
        prev_lower = c.is_lowercase();
    }

    result
}

/// Converts a string to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_lower = false;

    for c in s.chars() {
        if c.is_uppercase() && prev_lower {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
        prev_lower = c.is_lowercase();
    }

    result
}

/// Converts a string to PascalCase.
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Maps FIX field types to Rust types.
fn field_type_to_rust(field_type: &FieldType) -> &'static str {
    match field_type {
        FieldType::Int
        | FieldType::Length
        | FieldType::SeqNum
        | FieldType::NumInGroup
        | FieldType::TagNum
        | FieldType::DayOfMonth => "i64",
        FieldType::Float
        | FieldType::Qty
        | FieldType::Price
        | FieldType::PriceOffset
        | FieldType::Amt
        | FieldType::Percentage => "rust_decimal::Decimal",
        FieldType::Char => "char",
        FieldType::Boolean => "bool",
        FieldType::String
        | FieldType::MultipleCharValue
        | FieldType::MultipleStringValue
        | FieldType::Country
        | FieldType::Currency
        | FieldType::Exchange
        | FieldType::Language
        | FieldType::Pattern
        | FieldType::Tenor => "String",
        FieldType::MonthYear
        | FieldType::UtcTimestamp
        | FieldType::UtcTimeOnly
        | FieldType::UtcDateOnly
        | FieldType::LocalMktDate
        | FieldType::LocalMktTime
        | FieldType::TzTimeOnly
        | FieldType::TzTimestamp => "String",
        FieldType::Data | FieldType::XmlData => "Vec<u8>",
        FieldType::Reserved => "String",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_screaming_snake_case() {
        assert_eq!(to_screaming_snake_case("MsgType"), "MSG_TYPE");
        assert_eq!(to_screaming_snake_case("ClOrdID"), "CL_ORD_ID");
        assert_eq!(to_screaming_snake_case("BeginString"), "BEGIN_STRING");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("MsgType"), "msg_type");
        assert_eq!(to_snake_case("ClOrdID"), "cl_ord_id");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("new_order_single"), "NewOrderSingle");
        assert_eq!(to_pascal_case("execution_report"), "ExecutionReport");
    }

    #[test]
    fn test_generator_new() {
        let generator = CodeGenerator::new();
        assert!(generator.config.generate_fields);
        assert!(generator.config.generate_messages);
    }
}
