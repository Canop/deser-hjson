use {
    serde::{
        Deserialize,
    },
};

#[macro_use] mod common;

/// check we support braceless Hjson
#[test]
fn test_braceless() {
    #[derive(Debug, Deserialize)]
    struct T {
        field: Option<String>,
    }
    fn check(hjson: &str, field: Option<&str>) {
        println!("checking {hjson:?}");
        let t = deser_hjson::from_str::<T>(hjson).unwrap();
        assert_eq!(t.field, field.map(|s| s.to_string()));
    }
    check("{}", None);
    check(r#"{field:"value"}"#, Some("value"));
    check(r#"field:"value""#, Some("value"));
    check(r#"field:value"#, Some("value"));
    check(
        r#"
        field: value
        useless: line
        "#,
        Some("value")
    );
    check("    ", None);
    check("", None);
    check(
        r#"
            // just some comments
        "#,
        None,
    );
}

