use {
    serde::{
        Deserialize,
    },
};

#[macro_use] mod common;

/// check we fixed the bug #1
/// ("1" was resulting in an EOF)
#[test]
fn test_dont_need_trailing_spaces() {
    #[derive(Debug, Deserialize)]
    struct T {};
    deser_hjson::from_str::<T>("{}").unwrap();
    deser_hjson::from_str::<i32>("1").unwrap();
    deser_hjson::from_str::<f64>("1e-3").unwrap();
    deser_hjson::from_str::<f64>("-1.3").unwrap();
    deser_hjson::from_str::<Vec<u8>>("[]").unwrap();
}

#[test]
fn test_accept_trailing_spaces() {
    #[derive(Debug, Deserialize)]
    struct T {};
    deser_hjson::from_str::<T>("{}   ").unwrap();
    deser_hjson::from_str::<i32>("1 ").unwrap();
    deser_hjson::from_str::<f64>("1e-3 ").unwrap();
}

#[test]
fn test_choke_on_trailing_chars() {
    #[derive(Debug, Deserialize)]
    struct T {};
    assert!(deser_hjson::from_str::<T>("{}  e ").is_err());
    assert!(deser_hjson::from_str::<i32>("1 -").is_err());
    assert!(deser_hjson::from_str::<f64>("1e-3 e").is_err());
}

