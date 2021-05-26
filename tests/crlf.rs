use {
    deser_hjson,
    serde::Deserialize,
};

#[macro_use] mod common;

// check that CRLF are considered as LF
#[test]
fn test_crlf() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum Enum {
        A,
        B,
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct InnerStruct {
        txt: String,
        val: Enum,
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct OuterStruct {
        int: i32,
        float: f64,
        seq: Vec<String>,
        txt: Option<String>,
        structs: Vec<InnerStruct>,
    }
    let hjson_lf = r#"
    {
        # Some comments
        int: 44,

        seq: [
            "bla", // comments again
            '''
            some
            multiline
            string
            '''
            no comma
        ]
        float: 5.7, // comments too
        structs: [
            {
                txt: ""
                val: "A"
            }
            {
                val: "B"
                txt:
                    '''
                    also on
                    three
                    lines
                    '''
            }
        ]
    }
    "#;
    let hjson_crlf = hjson_lf.replace('\n', "\r\n");
    let hjson_crlf = &hjson_crlf;
    fn check(os: &OuterStruct) {
        assert_eq!(
            os.seq,
            vo!["bla", "some\nmultiline\nstring", "no comma"],
        );
        assert_eq!(os.int, 44);
        assert_eq!(
            os.structs[1].txt,
            "also on\nthree\nlines".to_owned(),
        );
    }
    let crlf = deser_hjson::from_str::<OuterStruct>(hjson_crlf).unwrap();
    let lf = deser_hjson::from_str::<OuterStruct>(hjson_lf).unwrap();
    check(&crlf);
    check(&lf);
    assert_eq!(
        crlf,
        lf,
    );
}

