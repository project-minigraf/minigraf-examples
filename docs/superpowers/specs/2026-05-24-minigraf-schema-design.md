# minigraf-schema Design Spec

**Date:** 2026-05-24
**Issue:** #3 — post-1.0: add optional schema validation library
**Status:** Approved

---

## Overview

`minigraf-schema` is a new workspace member crate providing optional schema validation for Minigraf databases. It lives alongside `minigraf-algorithms` in this workspace and will be published independently to crates.io.

Schema validation is intentionally outside the Minigraf core. Users who need it add `minigraf-schema` as a dependency; those who don't are unaffected.

---

## Architecture

Single workspace member at `minigraf-schema/`, following the `minigraf-algorithms` layout:

```
minigraf-schema/
  Cargo.toml          # publish = true, MIT OR Apache-2.0
  README.md
  src/
    lib.rs            # all public API, DSL parser, validator logic
  tests/
    schema.rs         # integration tests
```

Dependencies: `anyhow = "1.0"`, `minigraf = "1.1"`. DSL parser is hand-written — no parser combinator crates needed for this grammar.

---

## Entity Typing Model

Minigraf has no built-in entity type concept. `minigraf-schema` uses **attribute-based typing**: a designated type attribute (e.g. `:entity/type`) holds a keyword value (e.g. `:person`) that identifies which schema rules apply to that entity.

The type attribute and type value are both defined per schema block in the DSL — there is no global convention enforced by the crate.

---

## DSL

Schemas are defined as strings parsed at runtime. Each block covers one entity type.

```
entity :entity/type :person {
    required :name    String
    required :email   String
    optional :age     Integer
    optional :active  Boolean
    optional :org     Ref
}

entity :entity/type :project {
    required :name    String
    optional :owner   Ref
    optional :status  Keyword
}
```

**Syntax rules:**
- `entity <type-attr> <type-value> { ... }` opens a block
- `required <attribute> <type>` — attribute must be present with correct type
- `optional <attribute> <type>` — attribute may be absent; if present, must have correct type
- Type tokens: `String`, `Integer`, `Float`, `Boolean`, `Ref`, `Keyword`
- `Null` is excluded — a null value on a required attribute counts as missing
- Attribute names must start with `:`
- Multiple blocks in one string are allowed
- Whitespace and blank lines between tokens are ignored
- No comments in v1

**Parse errors:**
- Duplicate `entity` block (same type-attr + type-value pair)
- An attribute appearing in both `required` and `optional` in the same block
- Unrecognised type token
- Malformed attribute name (does not start with `:`)

---

## Public API

### Types

```rust
pub struct Schema { /* opaque */ }

pub enum ValueType {
    String,
    Integer,
    Float,
    Boolean,
    Ref,
    Keyword,
}

pub struct ValidationError {
    pub entity: String,   // the entity identifier as given (e.g. ":alice")
    pub kind: ValidationErrorKind,
}

pub enum ValidationErrorKind {
    MissingRequiredAttribute { attribute: String },
    TypeMismatch { attribute: String, expected: ValueType, actual: ValueType },
}
```

### Schema construction

```rust
impl Schema {
    pub fn parse(src: &str) -> Result<Self>
}
```

### Pre-transact validation

```rust
impl Schema {
    pub fn validate(&self, facts: &[(&str, &str, Value)]) -> Vec<ValidationError>
}
```

Facts are `(entity, attribute, value)` triples. `validate` is pure: no DB access, no I/O, infallible (returns `Vec`, not `Result<Vec>`).

Behaviour:
- Collects entity type assignments from the fact slice
- Applies matching schema rules per entity
- Entities with no matching schema block are silently ignored (open-world assumption)
- Returns all violations found, not just the first

### Audit

```rust
impl Schema {
    pub fn audit(&self, db: &Minigraf) -> Result<Vec<ValidationError>>
    pub fn audit_as_of(&self, db: &Minigraf, as_of: u64) -> Result<Vec<ValidationError>>
}
```

`audit()` delegates to `audit_as_of` using `db.current_tx_count()`.

The `as_of` parameter is the monotonic transaction counter from `Minigraf::current_tx_count()` — not a Unix timestamp. It maps directly to Minigraf's `:as-of N` Datalog clause.

Audit procedure per schema block:
1. Query all entities where `[?e <type-attr> <type-value>]` (with `:as-of N` if applicable)
2. For each entity, query all its current attribute/value pairs at that point in time
3. Check all required attributes are present with the correct type
4. Check all optional attributes, if present, have the correct type
5. Collect all violations across all entities

Retractions are respected: a retracted required attribute is treated as absent and produces a `MissingRequiredAttribute` violation.

---

## Temporal Interaction

Validation operates on one point in time:

- **Pre-transact (`validate`)**: checks the proposed fact slice only. It does not consider existing DB state — the caller is responsible for including all relevant facts for the entities being written.
- **Audit (`audit`)**: checks current DB state (latest transaction).
- **Audit as-of (`audit_as_of`)**: checks DB state at a past transaction. Retractions before that point are reflected; facts asserted after that point are invisible.

Schema definitions themselves are not stored in the DB and do not have temporal semantics.

---

## Testing

All tests in `tests/schema.rs` using `Minigraf::in_memory()`.

| Category | Cases |
|---|---|
| DSL parsing — valid | Well-formed single block; multiple blocks |
| DSL parsing — errors | Duplicate entity block; attribute in both required and optional; unrecognised type token |
| `validate` — passing | All required attrs present and correctly typed |
| `validate` — failing | Missing required attribute; type mismatch on required; type mismatch on optional (present but wrong type) |
| `validate` — open-world | Entity with no matching schema block produces no violations |
| `audit` — passing | DB state satisfies schema |
| `audit` — failing | Entity missing required attribute after retraction |
| `audit_as_of` | Entity valid at tx N; retraction at tx N+1; querying at N → no violations, at N+1 → violation |
