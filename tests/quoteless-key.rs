use {
    std::{
        collections::HashMap,
        fmt::Write,
    },
};

#[macro_use] mod common;

#[test]
fn test_quoteless_key() {
    // Build a hjson like this:
    //    s1:s1
    //    s2:s2
    // and check that it parses as a map even when
    // the si strings contain special characters
    let strings = [
        "this-one-is-easy",
        r#"@?;'"\/."#, // see https://github.com/Canop/deser-hjson/issues/9
        "abcd",
        "l'éléphant",
        "a=\"a\"",
        "z''''''",
        "こんにちわ",
    ];
    let mut hjson = String::new();
    for s in strings {
        writeln!(&mut hjson, "{}:{}", s, s).unwrap();
    }
    println!("Hjson:\n{}", &hjson);
    let map: HashMap<String, String> = deser_hjson::from_str(&hjson).unwrap();
    for s in strings {
        assert_eq!(map.get(s).unwrap(), s);
    }
}

#[test]
fn test_string_map_with_integer_key() {
    #[derive(serde::Deserialize, Debug)]
    struct S {
        map: HashMap<String, String>,
    }
    let hjson = r#"
    map: {
        1: "one",
        2: "two",
        3254884515488: "many",
    }
    "#;
    println!("Hjson:\n{}", &hjson);
    let s: S = deser_hjson::from_str(&hjson).unwrap();
    dbg!(&s);
    assert_eq!(s.map.get("1").unwrap(), "one");
    assert_eq!(s.map.len(), 3);
}
