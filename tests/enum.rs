use {
    deser_hjson::from_str,
    serde:: Deserialize,
};

#[macro_use] mod common;

#[test]
fn test_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let j = r#""Unit""#;
    let expected = E::Unit;
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{Newtype:1}"#;
    let expected = E::Newtype(1);
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"
    {
        Tuple : [ # Tuple variant
            1
            2
        ]
    }
    "#;
    let expected = E::Tuple(1, 2);
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"
    {
        # this variant is explitely defined
        Struct: {a:1}
    }"#;
    let expected = E::Struct { a: 1 };
    assert_eq!(expected, from_str(j).unwrap());
}

#[test]
fn test_quoteless_tag_variant() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        A,
        B,
    }
    let hjson = "B\n";
    assert_eq!(E::B, from_str(hjson).unwrap());
    let hjson = "B";
    assert_eq!(E::B, from_str(hjson).unwrap());
    #[derive(Deserialize, PartialEq, Debug)]
    struct S {
        e: E,
    }
    let hjson = r#"{
        e: B
    }"#;
    assert_eq!(S{e:E::B}, from_str(hjson).unwrap());
    let hjson = r#"{"e": "B"}"#;
    assert_eq!(S{e:E::B}, from_str(hjson).unwrap());
    let hjson = "{e:B}";
    assert_eq!(S{e:E::B}, from_str(hjson).unwrap());
}

#[test]
fn test_arr_struct_untagged() {
    // this enum is untagged: the variant is automatically recognized
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum Untagged {
        Int(u16),
        Float(f32),
        String(String),
        Array(Vec<String>),
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct InnerThing {
        name: String,
        untagged: Untagged,
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct OuterThing {
        outer_name: String,
        items: Vec<InnerThing>,
    }
    let hjson = r#"
        {
            outer_name: the thing
            items: [
                {
                    name: first item
                    untagged: "xterm -e \"nvim {file}\""
                }
                {
                    name: "also an \"item\""
                    untagged: ["bla", "et", "bla"]
                }
                {
                    name: third
                    untagged: 4
                }
                {
                    name: fourth
                    untagged: 4.3
                }
            ]
        }
    "#;
    let outer_thing = OuterThing {
        outer_name: "the thing".to_owned(),
        items: vec![
            InnerThing {
                name: "first item".to_owned(),
                untagged: Untagged::String("xterm -e \"nvim {file}\"".to_string()),
            },
            InnerThing {
                name: r#"also an "item""#.to_owned(),
                untagged: Untagged::Array(vo!["bla", "et", "bla"]),
            },
            InnerThing {
                name: "third".to_owned(),
                untagged: Untagged::Int(4),
            },
            InnerThing {
                name: "fourth".to_owned(),
                untagged: Untagged::Float(4.3),
            },
        ],
    };
    assert_eq!(outer_thing, from_str::<OuterThing>(hjson).unwrap());
}

