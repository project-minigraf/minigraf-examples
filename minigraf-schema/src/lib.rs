#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Schema validation for data stored in Minigraf.
//!
//! This crate lives outside `minigraf` core. Schema validation is an ecosystem
//! utility: useful for applications that want to enforce data contracts, but it
//! should not couple storage internals to application-level validation choices.

use std::collections::HashMap;

use anyhow::{Result, bail};
use minigraf::Value;

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

#[derive(Debug)]
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
#[derive(Debug)]
pub struct Schema {
    blocks: Vec<EntityBlock>,
}

// ── DSL parser ────────────────────────────────────────────────────────────────

impl Schema {
    /// Parse a schema definition string into a [`Schema`].
    ///
    /// # Example
    ///
    /// ```
    /// use minigraf_schema::Schema;
    ///
    /// let schema = Schema::parse(r#"
    ///     entity :entity/_type :person {
    ///         required :name  String
    ///         optional :age   Integer
    ///     }
    /// "#).unwrap();
    /// ```
    pub fn parse(src: &str) -> Result<Self> {
        let tokens = tokenize(src);
        let mut pos = 0;
        let mut blocks: Vec<EntityBlock> = Vec::new();

        while pos < tokens.len() {
            let t = next_tok(&tokens, &mut pos)?;
            if t != "entity" {
                bail!("expected 'entity', got {:?}", t);
            }

            let type_attr = parse_keyword(&tokens, &mut pos)?;
            let type_value = parse_keyword(&tokens, &mut pos)?;
            expect_tok(&tokens, &mut pos, "{")?;

            if blocks
                .iter()
                .any(|b| b.type_attr == type_attr && b.type_value == type_value)
            {
                bail!(
                    "duplicate entity block for {} {}",
                    type_attr,
                    type_value
                );
            }

            let mut required: HashMap<String, ValueType> = HashMap::new();
            let mut optional: HashMap<String, ValueType> = HashMap::new();

            loop {
                let t = next_tok(&tokens, &mut pos)?;
                match t {
                    "}" => break,
                    "required" => {
                        let attr = parse_keyword(&tokens, &mut pos)?;
                        let vtype = parse_value_type(&tokens, &mut pos)?;
                        if optional.contains_key(&attr) {
                            bail!(
                                "attribute {} appears in both required and optional",
                                attr
                            );
                        }
                        required.insert(attr, vtype);
                    }
                    "optional" => {
                        let attr = parse_keyword(&tokens, &mut pos)?;
                        let vtype = parse_value_type(&tokens, &mut pos)?;
                        if required.contains_key(&attr) {
                            bail!(
                                "attribute {} appears in both required and optional",
                                attr
                            );
                        }
                        optional.insert(attr, vtype);
                    }
                    other => bail!(
                        "expected 'required', 'optional', or '}}', got {:?}",
                        other
                    ),
                }
            }

            blocks.push(EntityBlock {
                type_attr,
                type_value,
                required,
                optional,
            });
        }

        Ok(Schema { blocks })
    }
}

fn tokenize(src: &str) -> Vec<String> {
    src.replace('{', " { ")
        .replace('}', " } ")
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

fn next_tok<'a>(tokens: &'a [String], pos: &mut usize) -> Result<&'a str> {
    if *pos >= tokens.len() {
        bail!("unexpected end of input");
    }
    let t = tokens[*pos].as_str();
    *pos += 1;
    Ok(t)
}

fn expect_tok(tokens: &[String], pos: &mut usize, expected: &str) -> Result<()> {
    let t = next_tok(tokens, pos)?;
    if t != expected {
        bail!("expected {:?}, got {:?}", expected, t);
    }
    Ok(())
}

fn parse_keyword(tokens: &[String], pos: &mut usize) -> Result<String> {
    let t = next_tok(tokens, pos)?;
    if !t.starts_with(':') {
        bail!("expected keyword starting with ':', got {:?}", t);
    }
    Ok(t.to_string())
}

fn parse_value_type(tokens: &[String], pos: &mut usize) -> Result<ValueType> {
    let t = next_tok(tokens, pos)?;
    match t {
        "String" => Ok(ValueType::String),
        "Integer" => Ok(ValueType::Integer),
        "Float" => Ok(ValueType::Float),
        "Boolean" => Ok(ValueType::Boolean),
        "Ref" => Ok(ValueType::Ref),
        "Keyword" => Ok(ValueType::Keyword),
        other => bail!(
            "unrecognised type {:?}; expected one of String, Integer, Float, Boolean, Ref, Keyword",
            other
        ),
    }
}

// ── Validation logic ──────────────────────────────────────────────────────────

impl Schema {
    /// Validate a slice of proposed facts against this schema.
    ///
    /// Facts are `(entity, attribute, value)` triples, matching the structure
    /// of a Minigraf `transact` call. The entity is any string identifier (e.g.
    /// `":alice"` or a UUID string).
    ///
    /// Entities whose type does not match any schema block are silently ignored
    /// (open-world assumption). Returns all violations found, not just the first.
    ///
    /// This function is pure — it performs no database access.
    pub fn validate(&self, facts: &[(&str, &str, Value)]) -> Vec<ValidationError> {
        // Build per-entity attribute maps from the fact slice.
        // Later facts for the same (entity, attribute) pair overwrite earlier ones.
        let mut entity_attrs: HashMap<&str, HashMap<&str, &Value>> = HashMap::new();
        for (entity, attr, value) in facts {
            entity_attrs.entry(entity).or_default().insert(attr, value);
        }

        let mut errors = Vec::new();

        for block in &self.blocks {
            for (entity, attrs) in &entity_attrs {
                let has_type = attrs
                    .get(block.type_attr.as_str())
                    .map(|v| matches!(v, Value::Keyword(kw) if *kw == block.type_value))
                    .unwrap_or(false);

                if !has_type {
                    continue;
                }

                check_block(block, entity, attrs, &mut errors);
            }
        }

        errors
    }
}

fn check_block(
    block: &EntityBlock,
    entity: &str,
    attrs: &HashMap<&str, &Value>,
    errors: &mut Vec<ValidationError>,
) {
    for (attr, expected) in &block.required {
        match attrs.get(attr.as_str()) {
            None => errors.push(ValidationError {
                entity: entity.to_string(),
                kind: ValidationErrorKind::MissingRequiredAttribute {
                    attribute: attr.clone(),
                },
            }),
            Some(value) => match value_type_of(value) {
                None => errors.push(ValidationError {
                    entity: entity.to_string(),
                    kind: ValidationErrorKind::MissingRequiredAttribute {
                        attribute: attr.clone(),
                    },
                }),
                Some(actual) if actual != *expected => errors.push(ValidationError {
                    entity: entity.to_string(),
                    kind: ValidationErrorKind::TypeMismatch {
                        attribute: attr.clone(),
                        expected: expected.clone(),
                        actual,
                    },
                }),
                Some(_) => {}
            },
        }
    }

    for (attr, expected) in &block.optional {
        if let Some(value) = attrs.get(attr.as_str()) {
            if let Some(actual) = value_type_of(value) {
                if actual != *expected {
                    errors.push(ValidationError {
                        entity: entity.to_string(),
                        kind: ValidationErrorKind::TypeMismatch {
                            attribute: attr.clone(),
                            expected: expected.clone(),
                            actual,
                        },
                    });
                }
            }
            // Value::Null on an optional attribute is treated as absent — no violation
        }
    }
}

fn value_type_of(v: &Value) -> Option<ValueType> {
    match v {
        Value::String(_) => Some(ValueType::String),
        Value::Integer(_) => Some(ValueType::Integer),
        Value::Float(_) => Some(ValueType::Float),
        Value::Boolean(_) => Some(ValueType::Boolean),
        Value::Ref(_) => Some(ValueType::Ref),
        Value::Keyword(_) => Some(ValueType::Keyword),
        Value::Null => None,
    }
}
