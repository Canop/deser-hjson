#[macro_use] mod common;

#[test]
fn issue_23() {
    use std::collections::HashMap;
    let with_newline = "foo: 3\n// bar: 5\n";
    assert_eq!(
        deser_hjson::from_str::<HashMap<_, _>>(with_newline).unwrap(),
        HashMap::from([
            ("foo".to_owned(), 3),
        ])
    );
    let without_newline = "foo: 3\n// bar: 5";
    assert_eq!(
        deser_hjson::from_str::<HashMap<_, _>>(without_newline).unwrap(),
        HashMap::from([
            ("foo".to_owned(), 3),
        ])
    );

}


