use {deser_hjson::from_str, serde::Deserialize, std::collections::HashMap};

#[macro_use]
mod common;

#[test]
fn test_weird_multiline() {
    // This is testing weird multilines things that are not documented in the spec.
    #[derive(Deserialize, PartialEq, Debug)]
    struct W {
        map: HashMap<String, String>,
    }
    let hjson = r#"{
           a: '''  bla    '''
           b: '''  bla<empty>
                  bli<empty>
                  hello<empty>
              '''
   }"#;
    let some_spaces = "    ";
    let hjson = hjson.replace("<empty>", some_spaces);
    println!("input: {:}", hjson);
    let mut map = HashMap::new();
    map.insert("a".to_owned(), "bla    ".to_owned());
    map.insert(
        "b".to_owned(),
        "bla".to_owned() + some_spaces + &"\nbli".to_owned() + some_spaces + &"\nhello".to_owned(),
    );
    assert_eq!(map, from_str(&hjson).unwrap());
}
