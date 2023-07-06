use {
    deser_hjson::from_reader,
    serde:: Deserialize,
};

#[macro_use] mod common;

#[test]
fn test_reader() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        a: i32,
        b: String,
    }
    let hjson = br#"{ a: 1, b: "2" }"#;
    let expected = Test {
        a: 1,
        b: "2".to_string(),
    };
    assert_eq!(expected, from_reader(&hjson[..]).unwrap());
}
