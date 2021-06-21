use {
    deser_hjson::from_str,
    serde:: Deserialize,
    glassbench::*,
};

static GIFTS: &[&str] = &[
    "{gift:null}",
    "{gift:false}",
    "{gift: true}",
    "{gift:'bar'}",
    r#"{gift:"bar"}"#,
    "{gift:42}",
    "{gift:42457811247}",
    "{gift:-42}",
    r#"{gift: "abcㅈ"}"#,
    "{gift:[15, -50]}",
    "{gift:[\"abc\"]}",
    r#"{gift:["abc", "another string"]}"#,
    r#" {
        gift: [
            "abc",
            "another string"
            and a third one (unquoted)
        ]
    }"#,
    "{gift:''}",
];

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
#[derive(Deserialize, PartialEq, Debug)]
struct WrappedGuess {
    gift: Guess,
}

fn bench_parse(bench: &mut Bench) {
    bench.task("guess wrapped", |task| {
        task.iter(|| {
            for hjson in GIFTS {
                let guessed = from_str::<WrappedGuess>(hjson)
                    .unwrap_or_else(|e| panic!("Parsing failed for {:?} : {}", hjson, e));
                pretend_used(guessed);
            }
        });
    });
}
glassbench!(
    "Parse",
    bench_parse,
);

