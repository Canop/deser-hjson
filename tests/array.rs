#[macro_use] mod common;

#[test]
fn array() {
    let arr: Vec<u8> = deser_hjson::from_str("[]").unwrap();
    assert_eq!(arr, vec![]);

    let arr: Vec<u8> = deser_hjson::from_str("[5, 3]").unwrap();
    assert_eq!(arr, vec![5, 3]);

    let arr: Vec<u8> = deser_hjson::from_str(" [ 5 ,\n  3  ] ").unwrap();
    assert_eq!(arr, vec![5, 3]);

    // A quoteless string goes til the end of the line.
    // It means than a string in an array must either be quoted
    // or go til the end of the line. The following array contains
    // only one element. I'm not making the spec :(
    let arr: Vec<String> = deser_hjson::from_str(r#"
        [a, 3] // not a comment
        ]
    "#
    ).unwrap();
    assert_eq!(arr, vec!["a, 3] // not a comment"]);

    // Another consequence of the quoteless string going til the end of the
    // line: the ']' is part of the string, and the array isn't closed
    assert!(deser_hjson::from_str::<Vec<String>>(r#"[abc, def]"#).is_err());

    let arr: Vec<String> = deser_hjson::from_str(r#"["abc", "def"]"#).unwrap();
    assert_eq!(arr, vec!["abc", "def"]);
}

