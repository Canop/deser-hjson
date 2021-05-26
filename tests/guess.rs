use {
    deser_hjson::from_str,
    serde:: Deserialize,
};

#[macro_use] mod common;

/// test precise primitive type guessing.
/// Note to users: be cautious with this, guessing types is
/// dangerous as Hjson is inherently ambiguous.
#[test]
fn test_guess_type() {
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum Guess {
        Bool(bool),
        U8(u8),
        I8(i8),
        U16(u16),
        I16(i16),
        U32(u32),
        I32(i32),
        U64(u64),
        I64(i64),
        F64(f64),
        Char(char),
        String(String),
        U16Array(Vec<u16>),
        I16Array(Vec<i16>),
        StrArray(Vec<String>),
    }
    assert_eq!(from_str::<Guess>("false").unwrap(), Guess::Bool(false));
    assert_eq!(from_str::<Guess>("-45").unwrap(), Guess::I8(-45));
    assert_eq!(from_str::<Guess>("45").unwrap(), Guess::U8(45));
    assert_eq!(from_str::<Guess>("453").unwrap(), Guess::U16(453));
    assert_eq!(from_str::<Guess>("-15453").unwrap(), Guess::I16(-15453));
    assert_eq!(from_str::<Guess>("39453").unwrap(), Guess::U16(39453));
    assert_eq!(from_str::<Guess>("-39453").unwrap(), Guess::I32(-39453));
    assert_eq!(from_str::<Guess>("139453").unwrap(), Guess::U32(139453));
    assert_eq!(from_str::<Guess>("34359738368").unwrap(), Guess::U64(34359738368));
    assert_eq!(from_str::<Guess>("-34359738368").unwrap(), Guess::I64(-34359738368));
    assert_eq!(from_str::<Guess>("-34e3").unwrap(), Guess::F64(-34000.0));
    assert_eq!(from_str::<Guess>("45.1").unwrap(), Guess::F64(45.1));
    assert_eq!(from_str::<Guess>("a").unwrap(), Guess::Char('a'));
    assert_eq!(from_str::<Guess>("abc").unwrap(), Guess::String("abc".to_owned()));
    assert_eq!(from_str::<Guess>("\"abc\"").unwrap(), Guess::String("abc".to_owned()));
    assert_eq!(from_str::<Guess>("[15, 50]").unwrap(), Guess::U16Array(vec![15, 50]));
    assert_eq!(from_str::<Guess>("[15, -50]").unwrap(), Guess::I16Array(vec![15, -50]));
    assert_eq!(from_str::<Guess>("[\"abc\"]").unwrap(), Guess::StrArray(vec!["abc".to_owned()]));
}

