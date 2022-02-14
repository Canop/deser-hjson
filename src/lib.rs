/*! A Serde 1.0 compatible deserializer for Hjson

```
use {
    deser_hjson::*,
    serde::Deserialize,
    std::collections::HashMap,
};
// This example comes from https://hjson.github.io/
let hjson = r#"
{
  // use #, // or /**/ comments,
  // omit quotes for keys
  key: 1
  // omit quotes for strings
  contains: everything on this line
  // omit commas at the end of a line
  cool: {
    foo: 1
    bar: 2
  }
  // allow trailing commas
  list: [
    1,
    2,
  ]
  // and use multiline strings
  realist:
    '''
    My half empty glass,
    I will fill your empty half.
    Now you are half full.
    '''
}
"#;
// we'll deserialize it into this struct:
#[derive(Deserialize, PartialEq, Debug)]
// optionally also add: #[serde(deny_unknown_fields)]
struct Example {
    key: i32,
    contains: Option<String>,
    cool: HashMap<String, u16>,
    list: Vec<usize>,
    realist: String,
    missing: Option<f64>,
}
let mut cool = HashMap::new();
cool.insert("foo".to_owned(), 1);
cool.insert("bar".to_owned(), 2);
let expected = Example {
    key: 1,
    contains: Some("everything on this line".to_owned()),
    cool,
    list: vec![1, 2],
    realist: "My half empty glass,\nI will fill your empty half.\nNow you are half full.".to_owned(),
    missing: None,
};
assert_eq!(expected, from_str(hjson).unwrap());

```
*/

mod de;
mod de_enum;
mod de_map;
mod de_number;
mod de_seq;
mod error;

pub use error::*;

/// deserialize the given string into a type implementing `Deserialize`
pub fn from_str<T>(s: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut deserializer = de::Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    deserializer.check_all_consumed()?;
    Ok(t)
}
