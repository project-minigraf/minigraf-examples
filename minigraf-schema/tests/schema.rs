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
