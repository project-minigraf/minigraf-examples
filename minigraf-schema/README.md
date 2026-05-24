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

// Set up a database with facts
let db = minigraf::Minigraf::in_memory().unwrap();
db.execute(r#"(transact [
    [:alice :entity/_type :person]
    [:alice :name "Alice"]
    [:alice :email "alice@example.com"]
])"#).unwrap();

// Audit current DB state
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
