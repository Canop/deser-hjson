use {
    serde::{
        de::Error,
        Deserialize,
        Deserializer,
    },
    std::collections::{
        BTreeMap,
        HashMap,
    },
};

#[macro_use] mod common;

// this example tries to test all the hard things of Hjson
#[test]
fn test_struct() {
    #[derive(PartialEq, Debug)]
    enum Enum {
        A,
        B,
    }
    // read "a" or "A" as A and "b" or "B" as B
    impl<'de> Deserialize<'de> for Enum {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where D: Deserializer<'de>
        {
            let s = String::deserialize(deserializer)?;
            let s = s.to_lowercase();
            match s.as_ref() {
                "a" => Ok(Enum::A),
                "b" => Ok(Enum::B),
                _ => Err(D::Error::custom(format!("unrecognized enum variant: {:?}", s))),
            }

        }
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        int: i32,
        float: f64,
        txt1: Option<String>,
        txt2: Option<String>,
        txt3: String,
        seq: Vec<String>,
        enum_map: HashMap<String, Enum>,
        numbers1: Vec<u32>,
        numbers2: Vec<i16>,
    }
    let hjson = r#"
    {
        # Hjson accepts several types of comments.
        /**
         * even the ugly java ones!
         * @WhatAmIDoingHere
         */

        // quotes around keys are optional
        "int": -1 # this comment goes to end of line
        txt2: a quoteless string : with a colon!
        txt3:
            '''
            you can have multiline strings
            and they're free of unexpected spacing
            '''

        // Hjson accepts trailing commas
        seq : [
            another quoteless string
            "b1\nb2",
            "c",
        ]

        enum_map: {
            "some key"    : a
            "another key" : B
        }

        # order of keys doesn't matter and you can
        # have a single value after a map
        float: -5.7

        numbers1: [ 559999,   87, 45,],
        numbers2: [
            -32
            876
            -111
            582
        ]
    }
    "#;
    let mut enum_map = HashMap::new();
    enum_map.insert("some key".to_owned(), Enum::A);
    enum_map.insert("another key".to_owned(), Enum::B);
    let expected = Test {
        int: -1,
        float: -5.7,
        txt1: None,
        txt2: Some("a quoteless string : with a colon!".to_owned()),
        txt3: "you can have multiline strings\nand they're free of unexpected spacing".to_owned(),
        seq: vo!["another quoteless string", "b1\nb2", "c"],
        enum_map,
        numbers1: vec![559999, 87, 45],
        numbers2: vec![-32, 876, -111, 582],
    };
    assert_eq!(expected, deser_hjson::from_str(hjson).unwrap());
}

#[test]
fn test_nested_everything() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Child {
        id: u64,
        enabled: bool,
        note: Option<String>,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Root {
        title: String,
        children: Vec<Child>,
        metadata: HashMap<String, String>,
        matrix: Vec<Vec<i32>>,
        empty_vec: Vec<String>,
        empty_map: BTreeMap<String, String>,
    }

    let hjson = r#"
    {
        title: example

        children: [
            {
                id: 1
                enabled: true
            }
            {
                id: 2
                enabled: false
                note: optional note
            }
        ]

        metadata: {
            a: b
            c: d
        }

        matrix: [
            [1, 2, 3]
            [4, 5, 6]
        ]

        empty_vec: []
        empty_map: {}
    }
    "#;

    let value: Root = deser_hjson::from_str(hjson).unwrap();

    assert_eq!(value.title, "example");
    assert_eq!(value.children.len(), 2);
    assert_eq!(value.children[1].note.as_deref(), Some("optional note"));
    assert_eq!(value.matrix[1][2], 6);
    assert!(value.empty_vec.is_empty());
    assert!(value.empty_map.is_empty());
}

#[test]
fn test_string_escapes() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Test {
        s: String,
    }

    let hjson = r#"
    {
        s: "line1\nline2\t\"quoted\"\\slash"
    }
    "#;

    let value: Test = deser_hjson::from_str(hjson).unwrap();

    assert_eq!(
        value.s,
        "line1\nline2\t\"quoted\"\\slash"
    );
}

#[test]
fn test_multiline_string_preserves_content() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Test {
        txt: String,
    }

    let hjson = r#"
    {
        txt:
            '''
            first
            second

            fourth
            '''
    }
    "#;

    let value: Test = deser_hjson::from_str(hjson).unwrap();

    assert_eq!(
        value.txt,
        "first\nsecond\n\nfourth"
    );
}

#[test]
fn test_numeric_boundaries() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Test {
        i8_min: i8,
        i8_max: i8,
        i16_min: i16,
        i16_max: i16,
        u32_max: u32,
    }

    let hjson = r#"
    {
        i8_min: -128
        i8_max: 127
        i16_min: -32768
        i16_max: 32767
        u32_max: 4294967295
    }
    "#;

    let value: Test = deser_hjson::from_str(hjson).unwrap();

    assert_eq!(value.i8_min, i8::MIN);
    assert_eq!(value.i8_max, i8::MAX);
    assert_eq!(value.i16_min, i16::MIN);
    assert_eq!(value.i16_max, i16::MAX);
    assert_eq!(value.u32_max, u32::MAX);
}

#[test]
fn test_default_fields() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Test {
        name: String,

        #[serde(default)]
        count: u32,

        #[serde(default)]
        tags: Vec<String>,
    }

    let hjson = r#"
    {
        name: hello
    }
    "#;

    let value: Test = deser_hjson::from_str(hjson).unwrap();

    assert_eq!(value.name, "hello");
    assert_eq!(value.count, 0);
    assert!(value.tags.is_empty());
}

#[test]
fn test_untagged_enum() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(untagged)]
    enum Value {
        Int(i32),
        Text(String),
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Test {
        a: Value,
        b: Value,
    }

    let hjson = r#"
    {
        a: 123
        b: hello
    }
    "#;

    let value: Test = deser_hjson::from_str(hjson).unwrap();

    assert_eq!(value.a, Value::Int(123));
    assert_eq!(value.b, Value::Text("hello".into()));
}

#[test]
fn test_comments_everywhere() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Test {
        a: i32,
        b: Vec<i32>,
    }

    let hjson = r#"
    {
        // before key
        a: 1 // after value

        b: [
            1 // first
            2 /* second */
            3
        ]
    }
    "#;

    let value: Test = deser_hjson::from_str(hjson).unwrap();

    assert_eq!(value.a, 1);
    assert_eq!(value.b, vec![1, 2, 3]);
}

#[test]
fn test_duplicate_keys_error() {
    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Test {
        value: i32,
    }

    let hjson = r#"
    {
        value: 1
        value: 2
    }
    "#;

    let result = deser_hjson::from_str::<Test>(hjson);

    assert!(result.is_err());
}

#[test]
fn test_integer_overflow_error() {
    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Test {
        value: i8,
    }

    let hjson = r#"
    {
        value: 128
    }
    "#;

    assert!(deser_hjson::from_str::<Test>(hjson).is_err());
}

#[test]
fn test_invalid_bool_error() {
    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Test {
        flag: bool,
    }

    let hjson = r#"
    {
        flag: maybe
    }
    "#;

    assert!(deser_hjson::from_str::<Test>(hjson).is_err());
}

#[test]
fn test_deeply_nested() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Level4 {
        value: String,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Level3 {
        inner: Level4,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Level2 {
        inner: Level3,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Level1 {
        inner: Level2,
    }

    let hjson = r#"
    {
        inner: {
            inner: {
                inner: {
                    value: success
                }
            }
        }
    }
    "#;

    let value: Level1 = deser_hjson::from_str(hjson).unwrap();

    assert_eq!(value.inner.inner.inner.value, "success");
}
