use {
    serde::{
        Deserialize,
    },
    std::collections::{
        HashMap,
    },
};

#[macro_use] mod common;


#[test]
fn test_evil_document() {
    #[derive(Debug, PartialEq)]
    enum Mode {
        Alpha,
        Beta,
    }

    impl<'de> Deserialize<'de> for Mode {
        fn deserialize<D>(d: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(d)?;
            match s.to_ascii_lowercase().as_str() {
                "alpha" => Ok(Mode::Alpha),
                "beta" => Ok(Mode::Beta),
                _ => Err(serde::de::Error::custom("bad mode")),
            }
        }
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Child {
        enabled: bool,
        mode: Mode,
        note: String,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Evil {
        int: i32,
        float: f64,

        plain: String,
        colon_string: String,
        url: String,

        multiline: String,

        seq: Vec<String>,

        child: Child,

        map: HashMap<String, String>,

        empty_vec: Vec<String>,
        empty_map: HashMap<String, String>,

        numbers: Vec<i32>,
    }

    let hjson = r#"
    {
        # ordering intentionally weird

        child: {
            enabled: true

            # enum from quoteless string
            mode: BeTa

            note: value with : colon but still one string
        }

        numbers: [
            -1
            0
            1
            1000000
        ]

        plain: ordinary quoteless string

        url: http://localhost:8080/api/v1?q=a:b

        multiline:
            '''
            line one
            line two

            line four
            '''

        map: {
            alpha: first value
            beta: second value
            gamma: value containing : colon
        }

        float: -123.75

        seq: [
            first item
            "second\nitem"
            third item
        ]

        empty_map: {}

        colon_string: this string contains : several : colons

        int: -42

        empty_vec: []
    }
    "#;

    let mut map = HashMap::new();
    map.insert("alpha".into(), "first value".into());
    map.insert("beta".into(), "second value".into());
    map.insert("gamma".into(), "value containing : colon".into());

    let expected = Evil {
        int: -42,
        float: -123.75,

        plain: "ordinary quoteless string".into(),

        colon_string:
            "this string contains : several : colons".into(),

        url:
            "http://localhost:8080/api/v1?q=a:b".into(),

        multiline:
            "line one\nline two\n\nline four".into(),

        seq: vec![
            "first item".into(),
            "second\nitem".into(),
            "third item".into(),
        ],

        child: Child {
            enabled: true,
            mode: Mode::Beta,
            note: "value with : colon but still one string".into(),
        },

        map,

        empty_vec: vec![],
        empty_map: HashMap::new(),

        numbers: vec![-1, 0, 1, 1_000_000],
    };

    let parsed: Evil = deser_hjson::from_str(hjson).unwrap();

    assert_eq!(parsed, expected);
}

#[test]
fn test_evil_transitions() {
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq)]
    enum Mode {
        Alpha,
        Beta,
    }

    impl<'de> Deserialize<'de> for Mode {
        fn deserialize<D>(d: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(d)?;
            match s.to_ascii_lowercase().as_str() {
                "alpha" => Ok(Mode::Alpha),
                "beta" => Ok(Mode::Beta),
                _ => Err(serde::de::Error::custom("bad mode")),
            }
        }
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Item {
        value: String,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Nested {
        list: Vec<Item>,
        map: HashMap<String, String>,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Child {
        enabled: bool,
        mode: Mode,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Evil {
        plain: String,
        url: String,
        multiline: String,

        child: Child,

        nested: Nested,

        seq: Vec<String>,

        matrix: Vec<Vec<i32>>,

        mixed: Vec<HashMap<String, String>>,

        trailing: Vec<i32>,
    }

    let hjson = r#"
    {
        plain: ordinary quoteless string

        url: http://localhost:8080/api?q=x:y

        multiline:
            '''
            alpha
            beta

            delta
            '''

        child: {
            enabled: true
            mode: BeTa
        }

        nested: {
            list: [
                {
                    value: one
                }

                {
                    value:
                        '''
                        two
                        '''
                }

                {
                    value: http://localhost:8080
                }

                {
                    value: x:y:z
                }
            ]

            map: {
                alpha: x:y
                beta: trueish
                gamma: nullify
                delta: localhost:8080
            }
        }

        seq: [
            one
            "two\nlines"

            three
        ]

        matrix: [
            [1, 2]
            [3, 4]
            [5, 6]
        ]

    # MIXED
        mixed: [ // list of maps
            {
                kind: alpha
                value: one
            }
            {
                kind: beta
                value:
                    '''
                    two
                    '''
            },
            {
                kind: "gamma" // comment
                value: http://example.com/a:b
            }
        ]

        trailing: [
            1,
            2
            3,
        ]
    } // } */
    //"#;

    let mut nested_map = HashMap::new();
    nested_map.insert("alpha".into(), "x:y".into());
    nested_map.insert("beta".into(), "trueish".into());
    nested_map.insert("gamma".into(), "nullify".into());
    nested_map.insert("delta".into(), "localhost:8080".into());

    let mut m1 = HashMap::new();
    m1.insert("kind".into(), "alpha".into());
    m1.insert("value".into(), "one".into());

    let mut m2 = HashMap::new();
    m2.insert("kind".into(), "beta".into());
    m2.insert("value".into(), "two".into());

    let mut m3 = HashMap::new();
    m3.insert("kind".into(), "gamma".into());
    m3.insert("value".into(), "http://example.com/a:b".into());

    let expected = Evil {
        plain: "ordinary quoteless string".into(),

        url: "http://localhost:8080/api?q=x:y".into(),

        multiline: "alpha\nbeta\n\ndelta".into(),

        child: Child {
            enabled: true,
            mode: Mode::Beta,
        },

        nested: Nested {
            list: vec![
                Item {
                    value: "one".into(),
                },
                Item {
                    value: "two".into(),
                },
                Item {
                    value: "http://localhost:8080".into(),
                },
                Item {
                    value: "x:y:z".into(),
                },
            ],
            map: nested_map,
        },

        seq: vec![
            "one".into(),
            "two\nlines".into(),
            "three".into(),
        ],

        matrix: vec![
            vec![1, 2],
            vec![3, 4],
            vec![5, 6],
        ],

        mixed: vec![m1, m2, m3],

        trailing: vec![1, 2, 3],
    };

    let actual: Evil = deser_hjson::from_str(hjson).unwrap();

    assert_eq!(actual, expected);
}
