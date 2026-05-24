# minigraf-schema Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `minigraf-schema` workspace crate that parses a schema DSL and validates Minigraf EAV facts against it — both pre-transact (pure, no DB) and via bi-temporal DB audit.

**Architecture:** Single `src/lib.rs` crate following the `minigraf-algorithms` pattern. The DSL is parsed into `EntityBlock` structs (internal); `validate()` is a pure function over a fact slice; `audit_as_of()` uses two Datalog queries per schema block — one to find typed entities, one to fetch all their attributes by UUID ref. Attribute variables in Minigraf queries return `Value::Keyword` for attribute names; entities come back as `Value::Ref(uuid)`.

**Tech Stack:** Rust 2024 edition, `minigraf = "1.1"`, `anyhow = "1.0"`. No parser combinator crates — the grammar fits a hand-written tokenizer + recursive descent parser.

---

### Task 1: Scaffold the crate

**Files:**
- Create: `minigraf-schema/Cargo.toml`
- Create: `minigraf-schema/src/lib.rs`
- Modify: `Cargo.toml` (workspace root, line 8)

- [ ] **Step 1: Add workspace member**

In `Cargo.toml` (root), change:
```toml
[workspace]
members = ["minigraf-algorithms"]
```
to:
```toml
[workspace]
members = ["minigraf-algorithms", "minigraf-schema"]
```

- [ ] **Step 2: Create `minigraf-schema/Cargo.toml`**

```toml
[package]
name = "minigraf-schema"
version = "0.1.0"
edition = "2024"
description = "Schema validation for Minigraf ecosystem crates"
license = "MIT OR Apache-2.0"
repository = "https://github.com/project-minigraf/minigraf-examples"
readme = "README.md"
keywords = ["graph", "minigraf", "schema", "validation"]
categories = ["data-structures", "database-interfaces"]

[dependencies]
anyhow = "1.0"
minigraf = "1.1"
```

- [ ] **Step 3: Create `minigraf-schema/src/lib.rs` stub**

```rust
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
```

- [ ] **Step 4: Verify the workspace compiles**

```bash
cargo check --workspace
```
Expected: no errors (the stub has no items yet, so there will be unused-import warnings — that is fine).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml minigraf-schema/Cargo.toml minigraf-schema/src/lib.rs
git commit -m "feat(minigraf-schema): scaffold crate and add to workspace"
```

---

### Task 2: Public types

**Files:**
- Modify: `minigraf-schema/src/lib.rs`

- [ ] **Step 1: Add types to `lib.rs`**

Append to `minigraf-schema/src/lib.rs` (after the `use` lines):

```rust
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
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo check -p minigraf-schema
```
Expected: compiles cleanly (possible dead-code warnings for the internal struct — fine).

- [ ] **Step 3: Commit**

```bash
git add minigraf-schema/src/lib.rs
git commit -m "feat(minigraf-schema): add public types and internal EntityBlock"
```

---

### Task 3: DSL parser

**Files:**
- Modify: `minigraf-schema/src/lib.rs`
- Create: `minigraf-schema/tests/schema.rs`

- [ ] **Step 1: Write failing parse tests**

Create `minigraf-schema/tests/schema.rs`:

```rust
use minigraf_schema::Schema;

// ── DSL parsing — valid ───────────────────────────────────────────────────────

#[test]
fn parse_single_block() {
    let src = r#"
        entity :entity/_type :person {
            required :name    String
            required :email   String
            optional :age     Integer
        }
    "#;
    assert!(Schema::parse(src).is_ok());
}

#[test]
fn parse_multiple_blocks() {
    let src = r#"
        entity :entity/_type :person {
            required :name String
        }
        entity :entity/_type :project {
            required :title String
        }
    "#;
    assert!(Schema::parse(src).is_ok());
}

#[test]
fn parse_all_value_types() {
    let src = r#"
        entity :entity/_type :thing {
            required :s  String
            required :i  Integer
            required :f  Float
            required :b  Boolean
            required :r  Ref
            required :k  Keyword
        }
    "#;
    assert!(Schema::parse(src).is_ok());
}

// ── DSL parsing — errors ──────────────────────────────────────────────────────

#[test]
fn parse_error_duplicate_entity_block() {
    let src = r#"
        entity :entity/_type :person { required :name String }
        entity :entity/_type :person { required :email String }
    "#;
    let err = Schema::parse(src).unwrap_err().to_string();
    assert!(err.contains("duplicate"), "expected 'duplicate' in error: {err}");
}

#[test]
fn parse_error_attribute_in_both_required_and_optional() {
    let src = r#"
        entity :entity/_type :person {
            required :name String
            optional :name String
        }
    "#;
    let err = Schema::parse(src).unwrap_err().to_string();
    assert!(
        err.contains("required") && err.contains("optional"),
        "expected conflict error, got: {err}"
    );
}

#[test]
fn parse_error_unrecognised_type_token() {
    let src = r#"
        entity :entity/_type :person {
            required :name Text
        }
    "#;
    let err = Schema::parse(src).unwrap_err().to_string();
    assert!(err.contains("Text") || err.contains("unrecognised"), "got: {err}");
}

#[test]
fn parse_error_attribute_missing_colon() {
    let src = r#"
        entity :entity/_type :person {
            required name String
        }
    "#;
    let err = Schema::parse(src).unwrap_err().to_string();
    assert!(err.contains("':'") || err.contains("keyword"), "got: {err}");
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test -p minigraf-schema 2>&1 | head -30
```
Expected: compile errors — `Schema` and `Schema::parse` do not exist yet.

- [ ] **Step 3: Implement the tokenizer and parser in `lib.rs`**

Append to `minigraf-schema/src/lib.rs`:

```rust
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
```

- [ ] **Step 4: Run parse tests and confirm they pass**

```bash
cargo test -p minigraf-schema 2>&1
```
Expected: all 7 parse tests pass.

- [ ] **Step 5: Commit**

```bash
git add minigraf-schema/src/lib.rs minigraf-schema/tests/schema.rs
git commit -m "feat(minigraf-schema): DSL tokenizer and parser with tests"
```

---

### Task 4: Pre-transact `validate()`

**Files:**
- Modify: `minigraf-schema/tests/schema.rs`
- Modify: `minigraf-schema/src/lib.rs`

- [ ] **Step 1: Add failing `validate` tests**

Append to `minigraf-schema/tests/schema.rs`:

```rust
// ── validate — passing ────────────────────────────────────────────────────────

#[test]
fn validate_passing_all_required_present_and_typed_correctly() {
    let schema = Schema::parse(r#"
        entity :entity/_type :person {
            required :name  String
            required :email String
            optional :age   Integer
        }
    "#).unwrap();

    use minigraf::Value;
    let facts: &[(&str, &str, Value)] = &[
        (":alice", ":entity/_type", Value::Keyword(":person".into())),
        (":alice", ":name",         Value::String("Alice".into())),
        (":alice", ":email",        Value::String("alice@example.com".into())),
        (":alice", ":age",          Value::Integer(30)),
    ];

    assert!(schema.validate(facts).is_empty());
}

#[test]
fn validate_passing_optional_attribute_absent() {
    let schema = Schema::parse(r#"
        entity :entity/_type :person {
            required :name  String
            optional :age   Integer
        }
    "#).unwrap();

    use minigraf::Value;
    let facts: &[(&str, &str, Value)] = &[
        (":alice", ":entity/_type", Value::Keyword(":person".into())),
        (":alice", ":name",         Value::String("Alice".into())),
        // :age intentionally absent
    ];

    assert!(schema.validate(facts).is_empty());
}

// ── validate — failing ────────────────────────────────────────────────────────

#[test]
fn validate_missing_required_attribute() {
    let schema = Schema::parse(r#"
        entity :entity/_type :person {
            required :name  String
            required :email String
        }
    "#).unwrap();

    use minigraf::Value;
    let facts: &[(&str, &str, Value)] = &[
        (":alice", ":entity/_type", Value::Keyword(":person".into())),
        (":alice", ":name",         Value::String("Alice".into())),
        // :email missing
    ];

    let errors = schema.validate(facts);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0].kind,
        minigraf_schema::ValidationErrorKind::MissingRequiredAttribute { attribute }
        if attribute == ":email"
    ));
    assert_eq!(errors[0].entity, ":alice");
}

#[test]
fn validate_type_mismatch_on_required_attribute() {
    let schema = Schema::parse(r#"
        entity :entity/_type :person {
            required :age Integer
        }
    "#).unwrap();

    use minigraf::Value;
    let facts: &[(&str, &str, Value)] = &[
        (":alice", ":entity/_type", Value::Keyword(":person".into())),
        (":alice", ":age",          Value::String("thirty".into())), // wrong type
    ];

    let errors = schema.validate(facts);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0].kind,
        minigraf_schema::ValidationErrorKind::TypeMismatch {
            attribute,
            expected: minigraf_schema::ValueType::Integer,
            actual: minigraf_schema::ValueType::String,
        }
        if attribute == ":age"
    ));
}

#[test]
fn validate_type_mismatch_on_optional_attribute_when_present() {
    let schema = Schema::parse(r#"
        entity :entity/_type :person {
            required :name String
            optional :age  Integer
        }
    "#).unwrap();

    use minigraf::Value;
    let facts: &[(&str, &str, Value)] = &[
        (":alice", ":entity/_type", Value::Keyword(":person".into())),
        (":alice", ":name",         Value::String("Alice".into())),
        (":alice", ":age",          Value::Boolean(true)), // wrong type
    ];

    let errors = schema.validate(facts);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0].kind,
        minigraf_schema::ValidationErrorKind::TypeMismatch {
            attribute,
            expected: minigraf_schema::ValueType::Integer,
            actual: minigraf_schema::ValueType::Boolean,
        }
        if attribute == ":age"
    ));
}

// ── validate — open-world ─────────────────────────────────────────────────────

#[test]
fn validate_open_world_entity_with_no_schema_block_produces_no_violations() {
    let schema = Schema::parse(r#"
        entity :entity/_type :person {
            required :name String
        }
    "#).unwrap();

    use minigraf::Value;
    // Entity with type :project — no schema block for it
    let facts: &[(&str, &str, Value)] = &[
        (":acme", ":entity/_type", Value::Keyword(":project".into())),
        // :name intentionally absent — but :project has no schema rules
    ];

    assert!(schema.validate(facts).is_empty());
}
```

- [ ] **Step 2: Run to confirm these tests fail**

```bash
cargo test -p minigraf-schema 2>&1 | grep -E "FAILED|error"
```
Expected: compile error — `Schema::validate` does not exist yet.

- [ ] **Step 3: Implement `validate()` and helpers in `lib.rs`**

Append to `minigraf-schema/src/lib.rs`:

```rust
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
```

- [ ] **Step 4: Run validate tests and confirm they pass**

```bash
cargo test -p minigraf-schema 2>&1
```
Expected: all tests pass (7 parse + 7 validate = 14 total).

- [ ] **Step 5: Commit**

```bash
git add minigraf-schema/src/lib.rs minigraf-schema/tests/schema.rs
git commit -m "feat(minigraf-schema): implement validate() with tests"
```

---

### Task 5: Audit API (`audit_as_of` and `audit`)

**Files:**
- Modify: `minigraf-schema/tests/schema.rs`
- Modify: `minigraf-schema/src/lib.rs`

**Background:** Minigraf's `execute()` returns `QueryResult::QueryResults { vars, results }` where each result row is `Vec<Value>`. Entity IDs come back as `Value::Ref(uuid)` even for entities transacted by keyword (e.g. `:alice` → UUID v5). Attribute names come back as `Value::Keyword(":attr-name")`. The audit uses a two-step approach per schema block:
1. `(query [:find ?e :as-of N :where [?e <type-attr> <type-value>]])` — find typed entities
2. Per entity UUID: `(query [:find ?a ?v :as-of N :where [#uuid "<uuid>" ?a ?v]])` — get all attributes

Combining these two into one query only returns the row matching the type constraint, not all attributes, so the two-step approach is required.

- [ ] **Step 1: Add failing audit tests**

Append to `minigraf-schema/tests/schema.rs`:

```rust
// ── audit — passing ───────────────────────────────────────────────────────────

#[test]
fn audit_passing_db_state_satisfies_schema() {
    let schema = Schema::parse(r#"
        entity :entity/_type :person {
            required :name  String
            required :email String
            optional :age   Integer
        }
    "#).unwrap();

    let db = minigraf::Minigraf::in_memory().unwrap();
    db.execute(r#"(transact [
        [:alice :entity/_type :person]
        [:alice :name "Alice"]
        [:alice :email "alice@example.com"]
        [:alice :age 30]
    ])"#).unwrap();

    let errors = schema.audit(&db).unwrap();
    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

// ── audit — failing ───────────────────────────────────────────────────────────

#[test]
fn audit_missing_required_attribute_after_retraction() {
    let schema = Schema::parse(r#"
        entity :entity/_type :person {
            required :name  String
            required :email String
        }
    "#).unwrap();

    let db = minigraf::Minigraf::in_memory().unwrap();
    db.execute(r#"(transact [
        [:alice :entity/_type :person]
        [:alice :name "Alice"]
        [:alice :email "alice@example.com"]
    ])"#).unwrap();

    // Retract :email — entity should now fail validation
    db.execute(r#"(retract [[:alice :email "alice@example.com"]])"#).unwrap();

    let errors = schema.audit(&db).unwrap();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0].kind,
        minigraf_schema::ValidationErrorKind::MissingRequiredAttribute { attribute }
        if attribute == ":email"
    ));
}

// ── audit_as_of ───────────────────────────────────────────────────────────────

#[test]
fn audit_as_of_entity_valid_before_retraction_invalid_after() {
    let schema = Schema::parse(r#"
        entity :entity/_type :person {
            required :name  String
            required :email String
        }
    "#).unwrap();

    let db = minigraf::Minigraf::in_memory().unwrap();
    db.execute(r#"(transact [
        [:alice :entity/_type :person]
        [:alice :name "Alice"]
        [:alice :email "alice@example.com"]
    ])"#).unwrap();
    let tx_before = db.current_tx_count();

    db.execute(r#"(retract [[:alice :email "alice@example.com"]])"#).unwrap();
    let tx_after = db.current_tx_count();

    let errors_before = schema.audit_as_of(&db, tx_before).unwrap();
    assert!(
        errors_before.is_empty(),
        "expected no errors at tx {tx_before}, got: {:?}",
        errors_before
    );

    let errors_after = schema.audit_as_of(&db, tx_after).unwrap();
    assert_eq!(
        errors_after.len(), 1,
        "expected 1 error at tx {tx_after}, got: {:?}",
        errors_after
    );
    assert!(matches!(
        &errors_after[0].kind,
        minigraf_schema::ValidationErrorKind::MissingRequiredAttribute { attribute }
        if attribute == ":email"
    ));
}
```

- [ ] **Step 2: Run to confirm tests fail**

```bash
cargo test -p minigraf-schema audit 2>&1 | grep -E "FAILED|error\[" | head -10
```
Expected: compile error — `Schema::audit` and `Schema::audit_as_of` do not exist yet.

- [ ] **Step 3: Implement `audit_as_of` and `audit` in `lib.rs`**

Append to `minigraf-schema/src/lib.rs`:

```rust
// ── Audit ─────────────────────────────────────────────────────────────────────

impl Schema {
    /// Check all entities in `db` against this schema at their current state.
    ///
    /// Equivalent to `audit_as_of(db, db.current_tx_count())`.
    pub fn audit(&self, db: &Minigraf) -> Result<Vec<ValidationError>> {
        self.audit_as_of(db, db.current_tx_count())
    }

    /// Check all entities in `db` against this schema as of transaction `as_of`.
    ///
    /// `as_of` is the monotonic transaction counter from
    /// [`Minigraf::current_tx_count`] — not a Unix timestamp. Retractions
    /// committed on or before `as_of` are reflected; facts committed after are
    /// invisible.
    pub fn audit_as_of(&self, db: &Minigraf, as_of: u64) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        for block in &self.blocks {
            // Step 1: find all entities of this type at this point in time.
            let type_query = format!(
                "(query [:find ?e :as-of {as_of} :where [?e {type_attr} {type_value}]])",
                as_of = as_of,
                type_attr = block.type_attr,
                type_value = block.type_value,
            );
            let result = db.execute(&type_query)?;
            let QueryResult::QueryResults { results: entity_rows, .. } = result else {
                bail!("expected QueryResults from entity type query");
            };

            for row in entity_rows {
                let entity_value = match row.into_iter().next() {
                    Some(v) => v,
                    None => continue,
                };

                let entity_str = entity_display(&entity_value);
                let entity_ref = match entity_datalog_ref(&entity_value) {
                    Some(s) => s,
                    None => continue,
                };

                // Step 2: get all attributes for this entity at this point in time.
                let attr_query = format!(
                    "(query [:find ?a ?v :as-of {as_of} :where [{entity_ref} ?a ?v]])",
                    as_of = as_of,
                    entity_ref = entity_ref,
                );
                let attr_result = db.execute(&attr_query)?;
                let QueryResult::QueryResults { results: attr_rows, .. } = attr_result else {
                    bail!("expected QueryResults from attribute query");
                };

                // Build attribute → value map. Last writer wins for duplicates.
                let mut attrs: HashMap<String, Value> = HashMap::new();
                for attr_row in attr_rows {
                    if let [Value::Keyword(attr), value] = attr_row.as_slice() {
                        attrs.insert(attr.clone(), value.clone());
                    }
                }

                // Reuse check_block logic — adapt types for owned data.
                check_block_owned(block, &entity_str, &attrs, &mut errors);
            }
        }

        Ok(errors)
    }
}

fn check_block_owned(
    block: &EntityBlock,
    entity: &str,
    attrs: &HashMap<String, Value>,
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
        }
    }
}

/// Format a `Value` as a human-readable entity identifier for error reporting.
fn entity_display(v: &Value) -> String {
    match v {
        Value::Keyword(k) => k.clone(),
        Value::Ref(uuid) => uuid.to_string(),
        other => format!("{other:?}"),
    }
}

/// Format a `Value` as the Datalog inline entity reference used in query strings.
///
/// Returns `None` for value types that cannot appear in the entity position.
fn entity_datalog_ref(v: &Value) -> Option<String> {
    match v {
        Value::Keyword(k) => Some(k.clone()),
        Value::Ref(uuid) => Some(format!("#uuid \"{uuid}\"")),
        _ => None,
    }
}
```

- [ ] **Step 4: Run all tests and confirm they pass**

```bash
cargo test -p minigraf-schema 2>&1
```
Expected: all 16 tests pass (7 parse + 6 validate + 3 audit).

- [ ] **Step 5: Commit**

```bash
git add minigraf-schema/src/lib.rs minigraf-schema/tests/schema.rs
git commit -m "feat(minigraf-schema): implement audit_as_of and audit with tests"
```

---

### Task 6: README and final workspace check

**Files:**
- Create: `minigraf-schema/README.md`

- [ ] **Step 1: Create `minigraf-schema/README.md`**

```markdown
# minigraf-schema

Schema validation for [Minigraf](https://crates.io/crates/minigraf) databases.

Defines entity types and attribute constraints using a small DSL. Validates
proposed facts before a `transact` call, or audits existing database state
(with optional time-travel via transaction counter).

## Installation

```toml
[dependencies]
minigraf-schema = "0.1"
```

## Quick Start

```rust
use minigraf::Value;
use minigraf_schema::Schema;

let schema = Schema::parse(r#"
    entity :entity/_type :person {
        required :name  String
        required :email String
        optional :age   Integer
    }
"#).unwrap();

// Pre-transact validation (pure, no DB access)
let facts = &[
    (":alice", ":entity/_type", Value::Keyword(":person".into())),
    (":alice", ":name",         Value::String("Alice".into())),
    (":alice", ":email",        Value::String("alice@example.com".into())),
];
let violations = schema.validate(facts);

// Audit current DB state
let db = minigraf::Minigraf::in_memory().unwrap();
let violations = schema.audit(&db).unwrap();

// Audit as of a past transaction
let tx = db.current_tx_count();
let violations = schema.audit_as_of(&db, tx).unwrap();
```

## DSL

Each `entity` block declares which Minigraf attribute signals the entity type,
what value it must hold, and which attributes are required or optional on
matching entities.

```
entity :entity/_type :person {
    required :name    String
    required :email   String
    optional :age     Integer
    optional :active  Boolean
    optional :org     Ref
}
```

Supported types: `String`, `Integer`, `Float`, `Boolean`, `Ref`, `Keyword`.

Entities whose type does not match any block are silently ignored
(open-world assumption).
```

- [ ] **Step 2: Run full workspace test suite**

```bash
cargo test --workspace 2>&1
```
Expected: all tests pass across `minigraf-examples`, `minigraf-algorithms`, and `minigraf-schema`.

- [ ] **Step 3: Commit**

```bash
git add minigraf-schema/README.md
git commit -m "docs(minigraf-schema): add README with DSL reference and quick start"
```
