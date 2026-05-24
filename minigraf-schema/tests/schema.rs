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
