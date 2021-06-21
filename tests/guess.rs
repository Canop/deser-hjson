use {
    deser_hjson::from_str,
    serde:: Deserialize,
};

#[macro_use] mod common;

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
    String(Option<String>),
    U16Array(Vec<u16>),
    I16Array(Vec<i16>),
    StrArray(Vec<String>),
}
fn string(s: &str) -> Guess {
    Guess::String(Some(s.to_owned()))
}
fn guess(hjson: &str, answer: Guess) {
    let guessed = from_str::<Guess>(hjson)
        .unwrap_or_else(|e| panic!("Parsing failed for {:?} : {}", hjson, e));
    if guessed != answer {
        panic!("Wrong guess for {:?} : guessed {:?} instead of {:?}", hjson, guessed, answer);
    }
}
#[derive(Deserialize, PartialEq, Debug)]
struct WrappedGuess {
    gift: Guess,
}
fn guess_wrapped(hjson: &str, answer: Guess) {
    let wrapped = from_str::<WrappedGuess>(hjson)
        .unwrap_or_else(|e| panic!("Parsing failed for {:?} : {}", hjson, e));
    let guessed = wrapped.gift;
    if guessed != answer {
        panic!("Wrong guess for {:?} : guessed {:?} instead of {:?}", hjson, guessed, answer);
    }
}




/// test precise primitive type guessing.
/// Note to users: be cautious with this, guessing types is
/// dangerous as Hjson is inherently ambiguous.
#[test]
fn test_guess_type() {
    guess("false", Guess::Bool(false));
    guess("-45", Guess::I8(-45));
    guess("45", Guess::U8(45));
    guess("453", Guess::U16(453));
    guess("-15453", Guess::I16(-15453));
    guess("39453", Guess::U16(39453));
    guess("-39453", Guess::I32(-39453));
    guess("139453", Guess::U32(139453));
    guess("34359738368", Guess::U64(34359738368));
    guess("-34359738368", Guess::I64(-34359738368));
    guess("-34e3", Guess::F64(-34000.0));
    guess("45.1", Guess::F64(45.1));
    guess("a", Guess::Char('a'));
    guess("abcㅈ", string("abcㅈ"));
    guess("\"abc\"", string("abc"));
    guess("'abc'", string("abc"));
    guess("''", string(""));
    guess("\"\"", string(""));
    guess("null", Guess::String(None));
    guess("[15, 50]", Guess::U16Array(vec![15, 50]));
    guess("[15, -50]", Guess::I16Array(vec![15, -50]));
    guess("[\"abc\"]", Guess::StrArray(vo!["abc"]));
    guess("[\"\"]", Guess::StrArray(vo![""]));
}

/// check a few tricky guesses, mostly the problems related
/// to braces on the line of what looks like a quoteless string
/// (see issue #3)
#[test]
fn test_wrapped_guess() {
    guess_wrapped("{gift:null}", Guess::String(None));
    guess_wrapped("{gift:false}", Guess::Bool(false));
    guess_wrapped("{gift: true}", Guess::Bool(true));
    guess_wrapped("{gift:'bar'}", string("bar"));
    guess_wrapped(r#"{gift:"bar"}"#, string("bar"));
    guess_wrapped("{gift:42}", Guess::U8(42));
    guess_wrapped(
        r#" {
            gift: [
                "abc",
                "another string"
                and a third one (unquoted)
            ]
        }"#,
        Guess::StrArray(vo![
                "abc",
                "another string",
                "and a third one (unquoted)",
        ]),
    );
}
