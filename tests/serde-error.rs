use serde::Deserialize;

#[macro_use] mod common;

/// this test checks we're converting serde message errors
/// to errors with some (approximate) position
#[test]
fn test_no_raw_serde_error() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Data {
        name: String,
        pos: Vec<Pos>,
    }
    #[derive(Deserialize, PartialEq, Debug)]
    enum Pos {
        Int(u8),
        Point(u8, u8),
    }
    let hjson_strings = vec![
        r#"{}"#,
        r#"[]"#,
        r#""#,
        r#"a"#,
        r#"{name: "albert", name: "alfred"}"#,
        r#"{pos: "not a pos"}"#,
        r#"{name: "", pos: [{3}]}"#,
        r#"{name: "", pos: [{(3, 4)}]}"#,
    ];
    for hjson in &hjson_strings {
        match deser_hjson::from_str::<Data>(hjson) {
            Ok(_) => {
                panic!("Unexpected Success deserializing {:?}", hjson);
            }
            Err(e@deser_hjson::Error::RawSerde(_)) => {
                panic!("Unexpected Raw Serde Error: {:?}", e);
            }
            Err(deser_hjson::Error::Serde{..}) => {},
            Err(deser_hjson::Error::Syntax{..}) => {},
        }
    }
}

