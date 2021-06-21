use serde_value::Value;

#[macro_use] mod common;

/// check we fixed the bug #3
#[test]
fn test_member_values() {
    // These values were problematic
    deser_hjson::from_str::<Value>("{foo:null}").unwrap();
    deser_hjson::from_str::<Value>("{foo:false}").unwrap();
    deser_hjson::from_str::<Value>("{foo:true}").unwrap();
    deser_hjson::from_str::<Value>("{foo:'bar'}").unwrap();
    // Also check some already working values
    deser_hjson::from_str::<Value>(r#"{foo:"bar"}"#).unwrap();
    deser_hjson::from_str::<Value>("{foo:42}").unwrap();
}
