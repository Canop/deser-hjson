use {
    std::collections::HashMap,
};

#[macro_use] mod common;

#[test]
fn test_quoteless_key() {
    let hjson = r#"@?;'"\/.: value"#; // see https://github.com/Canop/deser-hjson/issues/9
    let map: HashMap<String, String> = deser_hjson::from_str(hjson).unwrap();
    dbg!(&map);
    let (key, value) = map.iter().next().unwrap();
    assert_eq!(key, r#"@?;'"\/."#);
    assert_eq!(value, "value");
}

