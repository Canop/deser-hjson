use {
    deser_hjson::from_str,
    serde:: Deserialize,
    std::collections::HashMap,
};

#[macro_use] mod common;

/// Check that preceding and trailing whitespaces in
/// quoteless strings are ignored
#[test]
fn preceding_and_trailing_whitespaces_in_quoteless() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct W {
        map: HashMap<String, String>,
        single: String,
        arr: Vec<String>,
    }
    let hjson = "{\n\
        map: {\n\
            \" \":  0 \n\
            pi: 3.14    \t\n\
            τ:\t\t\t6.28  \n\
            'τ/2':   π  \n\
            /:  some tabs\t\t\t\n\
        },
        single: \t z -. \n\
        arr: [\n\
            \t bah\n\
               zz   \n\
        ]\n\
    }";
    let w: W = from_str(hjson).unwrap();
    dbg!(&w);
    let value = W {
        map: mo!{
             "τ/2": "π",
             "/": "some tabs",
             " ": "0",
             "pi": "3.14",
             "τ": "6.28",
        },
        single: "z -.".to_string(),
        arr: vo![
            "bah",
            "zz",
        ],
    };
    assert_eq!(value, w);
}

