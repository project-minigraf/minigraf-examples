#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Schema validation for data stored in Minigraf.
//!
//! This crate lives outside `minigraf` core. Schema validation is an ecosystem
//! utility: useful for applications that want to enforce data contracts, but it
//! should not couple storage internals to application-level validation choices.

use std::collections::HashMap;

use anyhow::{Result, bail};
use minigraf::{Minigraf, QueryResult, Value};

/// The expected type of a Minigraf attribute value, as declared in a schema block.
///
/// Mirrors the discriminants of [`minigraf::Value`]. `Null` is excluded — a null
/// value on a required attribute is treated as missing.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// [`minigraf::Value::String`]
    String,
    /// [`minigraf::Value::Integer`]
    Integer,
    /// [`minigraf::Value::Float`]
    Float,
    /// [`minigraf::Value::Boolean`]
    Boolean,
    /// [`minigraf::Value::Ref`]
    Ref,
    /// [`minigraf::Value::Keyword`]
    Keyword,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::String => write!(f, "String"),
            ValueType::Integer => write!(f, "Integer"),
            ValueType::Float => write!(f, "Float"),
            ValueType::Boolean => write!(f, "Boolean"),
            ValueType::Ref => write!(f, "Ref"),
            ValueType::Keyword => write!(f, "Keyword"),
        }
    }
}

/// A schema violation found by [`Schema::validate`] or [`Schema::audit_as_of`].
#[derive(Debug)]
pub struct ValidationError {
    /// The entity on which the violation was found.
    ///
    /// For keyword entities (`:alice`) this is the keyword string.
    /// For UUID entities this is the UUID string.
    pub entity: String,
    /// The kind of violation.
    pub kind: ValidationErrorKind,
}

/// The kind of schema violation.
#[derive(Debug)]
pub enum ValidationErrorKind {
    /// A `required` attribute was absent or had a `null` value.
    MissingRequiredAttribute {
        /// The attribute name, e.g. `":name"`.
        attribute: String,
    },
    /// An attribute was present but had the wrong value type.
    TypeMismatch {
        /// The attribute name.
        attribute: String,
        /// The type declared in the schema.
        expected: ValueType,
        /// The type of the value that was actually present.
        actual: ValueType,
    },
}

// ── Internal representation ───────────────────────────────────────────────────

struct EntityBlock {
    type_attr: String,
    type_value: String,
    required: HashMap<String, ValueType>,
    optional: HashMap<String, ValueType>,
}

/// A parsed schema definition.
///
/// Construct with [`Schema::parse`], then call [`Schema::validate`] or
/// [`Schema::audit`] / [`Schema::audit_as_of`].
pub struct Schema {
    blocks: Vec<EntityBlock>,
}
