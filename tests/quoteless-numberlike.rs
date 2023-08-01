use serde::Deserialize;

#[macro_use]
mod common;

#[test]
fn test_quoteless_number_like() {
    let hjson = r#"{
        value: "a"
        number: 10
        string: abc
        hex_but_string: 0x32
        sameline1_number: 10, sameline1_string: abc
        sameline2_string1: hello, sameline2_string2: abc
    }"#;
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        value: String,
        // Certain fields are ignored to force the parser to handle
        // them without types.
        sameline1_string: String,
        sameline2_string2: Option<String>,
    }
    let expected = Test {
        value: "a".to_string(),
        sameline1_string: "abc".to_string(),
        sameline2_string2: None,
    };
    assert_eq!(expected, deser_hjson::from_str(&hjson).unwrap());
}

#[test]
fn test_quoteless_number_like_with_space() {
    let hjson = r#"{
        value: "a"
        string_with_space: 10 apples
        sameline2_string: 30 19, sameline2_string2: abc
    }"#;
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        value: String,
        // Certain fields are ignored to force the parser to handle
        // them without types.
        sameline2_string2: Option<String>,
    }
    let expected = Test {
        value: "a".to_string(),
        sameline2_string2: None,
    };
    assert_eq!(expected, deser_hjson::from_str(&hjson).unwrap());
}
