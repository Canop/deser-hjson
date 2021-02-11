
#[macro_use] mod common;

/// check we fail when the data is invalid
#[test]
fn test_bad_format() {
    assert!(deser_hjson::from_str::<u32>("-1").is_err());
    assert!(deser_hjson::from_str::<i32>("1e-3").is_err());
    assert!(deser_hjson::from_str::<f64>("1e-3e-5").is_err());
}

