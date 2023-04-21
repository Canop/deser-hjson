use {
    serde::{
        de::Error,
        Deserialize,
        Deserializer,
    },
    std::collections::HashMap,
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
    };
    assert_eq!(expected, deser_hjson::from_str(hjson).unwrap());
}

