use {
    deser_hjson::from_str,
    serde:: Deserialize,
};

#[macro_use] mod common;

#[test]
fn test_weird_multiline_strings() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct W {
        a: String,
        b: String,
        c: String,
        d: String,
        e: String,
        f: String,
    }
    let hjson = r#"{
        a: '''all on the key line'''
        b: '''line 1
                line 2'''
        c: '''
line 1
    line 2'''
        d: ''' line 1
        line 2
    line 3'''
        e: '''
            line 1
                line 2
             '''
        f:
        '''
        line 1
         line 2
        line 3
        '''
}
    "#;
    let value = W {
        a: "all on the key line".to_string(),
        b: "line 1\n     line 2".to_string(),
        c: "line 1\nline 2".to_string(),
        d: "line 1\nline 2\nline 3".to_string(),
        e: " line 1\n     line 2\n  ".to_string(),
        f: "line 1\n line 2\nline 3".to_string(),
    };
    assert_eq!(value, from_str(hjson).unwrap());
}

/// check issue #19 https://github.com/Canop/deser-hjson/issues/19
#[test]
fn issue_19() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct W {
        title: String,
        repo: String,
        target: String,
        prefix: String,
        meta_image: String,
        description: String,
        utilities: Vec<String>,
    }
    let hjson = r#"
{
  title : CodeStage example
  repo : https://github.com/shi-yan/codestage
  # need to have the slash
  prefix : ""
  target : dist
  url : "http://localhost:8000"
  meta_image : meta.png
  description :
    '''CodeStage is a static site generator to build JS playground demos.'''
  utilities : [ "khronos",  "tdl" ]
}
    "#;
    let value = W {
        title : "CodeStage example".to_string(),
        repo : "https://github.com/shi-yan/codestage".to_string(),
        prefix : "".to_string(),
        target : "dist".to_string(),
        meta_image : "meta.png".to_string(),
        description : "CodeStage is a static site generator to build JS playground demos.".to_string(),
        utilities : vo!["khronos", "tdl"],
    };
    assert_eq!(value, from_str(hjson).unwrap());
}
