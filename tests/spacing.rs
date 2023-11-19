use {
    serde::Deserialize,
};

#[macro_use] mod common;

// look for problems with tab spacing
#[test]
fn test_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Verb {
        key: String,
        execution: String,
    }
    let hjson = r#"
    {
      key       : up
      execution	: ":line_up_no_cycle" // there's a tab before the colon
    }
    "#;
    let expected = Verb {
        key: "up".to_string(),
        execution: ":line_up_no_cycle".to_string(),
    };
    assert_eq!(expected, deser_hjson::from_str(hjson).unwrap());
}

// https://github.com/Canop/deser-hjson/issues/18
#[test]
fn test_bool_after_whitespace() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Bool {
        ping: bool,
    }
    let b: Bool = deser_hjson::from_str("ping:true\n ").unwrap();
    assert_eq!(b, Bool { ping: true });

    let b: Bool = deser_hjson::from_str(r#"ping: true"#).unwrap();
    assert_eq!(b, Bool { ping: true });
}

/// cf https://github.com/Canop/deser-hjson/issues/20
#[test]
fn issue_20() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Thing {
        dirs: Vec<u8>,
    }
    let hjson = "{\n    dirs: [\n    ]\n}";
    let _: Thing = deser_hjson::from_str(hjson).unwrap();
}
