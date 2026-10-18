use synapse_compiler::parser::parse_and_validate;

#[test]
fn test_duplicate_field_ids_rejected() {
    let idl = r#"
        namespace test.duplicate;

        struct Item {
            1: string a;
            1: string b;
        }
    "#;

    let res = parse_and_validate(idl);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("duplicate field ID 1"));
}

#[test]
fn test_unresolved_type_rejected() {
    let idl = r#"
        namespace test.unresolved;

        struct Item {
            1: NonExistentType field;
        }
    "#;

    let res = parse_and_validate(idl);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("Undefined type 'NonExistentType'"));
}

#[test]
fn test_zero_field_id_rejected() {
    let idl = r#"
        namespace test.zero_id;

        struct Item {
            0: string invalid;
        }
    "#;

    let res = parse_and_validate(idl);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("invalid field id 0"));
}

#[test]
fn test_duplicate_enum_values_rejected() {
    let idl = r#"
        namespace test.enum_dup;

        enum Mode {
            Alpha = 1,
            Beta = 1,
        }
    "#;

    let res = parse_and_validate(idl);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("duplicate variant value 1"));
}
