/*! A Serde deserializer for Hjson

```
use {
    deser_hjson::*,
    serde::Deserialize,
    std::collections::HashMap,
};
// This example comes from https://hjson.github.io/
let hjson = r#"
// use #, // or /**/ for comments,
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
"#;
// we'll deserialize it into this struct:
#[derive(Deserialize, PartialEq, Debug)]
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

/// Deserialize an instance of type `T` from a reader of Hjson text
///
/// # Example
///
/// ```
/// use serde::Deserialize;
/// use std::io::Cursor;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// // The type of `j` is `Cursor` which implements the `Read` trait
/// let j = Cursor::new("
///     fingerprint: 0xF9BA143B95FF6D82
///     location: Menlo Park, CA
/// ");
///
/// let u: User = deser_hjson::from_reader(j).unwrap();
/// println!("{:#?}", u);
/// ```
pub fn from_reader<R, T>(mut reader: R) -> Result<T>
where
    R: std::io::Read,
    T: serde::de::DeserializeOwned,
{
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    from_slice(&buf)
}


/// Deserialize an instance of type `T` from bytes of Hjson text
///
/// # Example
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// // The type of `j` is `&[u8]`
/// let j = b"
///     fingerprint: 0xF9BA143B95FF6D82
///     location: Menlo Park, CA
/// ";
///
/// let u: User = deser_hjson::from_slice(j).unwrap();
/// println!("{:#?}", u);
/// ```
pub fn from_slice<T>(bytes: &[u8]) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let s = std::str::from_utf8(bytes)?;
    from_str(s)
}


/// Deserialize an instance of type `T` from a string of Hjson text
///
/// # Example
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     hands: Option<u16>,
///     location: String,
/// }
///
/// // The type of `j` is `&str`
/// let j = "
///     hands: 2
///     location: Menlo Park, CA
/// ";
///
/// let u: User = deser_hjson::from_str(j).unwrap();
/// println!("{:#?}", u);
/// ```
pub fn from_str<T>(s: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut deserializer = de::Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    deserializer.check_all_consumed()?;
    Ok(t)
}
