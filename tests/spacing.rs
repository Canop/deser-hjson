use {
    deser_hjson,
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

