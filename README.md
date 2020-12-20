[![MIT][s2]][l2] [![Latest Version][s1]][l1] [![docs][s3]][l3] [![Chat on Miaou][s4]][l4]

[s1]: https://img.shields.io/crates/v/deser-hjson.svg
[l1]: https://crates.io/crates/deser-hjson

[s2]: https://img.shields.io/badge/license-MIT-blue.svg
[l2]: LICENSE

[s3]: https://docs.rs/deser-hjson/badge.svg
[l3]: https://docs.rs/deser-hjson/

[s4]: https://miaou.dystroy.org/static/shields/room.svg
[l4]: https://miaou.dystroy.org/3768

# deser_hjson

This is a Serde 1.0 compatible deserializer for [Hjson](https://hjson.github.io/), tailored for derive powered deserialization.

It's a work-in-progress, having been tested only minimally for now.

If you're interested in using this deserializer, or notice a problem, please come and tell me on [Miaou](https://miaou.dystroy.org/3768).

## Example

This Hjson document comes from [Hjson's introduction](https://hjson.github.io/)

```rust
use {
    deser_hjson::*,
    serde::Deserialize,
    std::collections::HashMap,
};
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
// Here's the deserialization and the equality check:
assert_eq!(expected, from_str(hjson).unwrap());
```

## Limits

### Colons in quoteless strings

Quoteless strings in Hjson end with the line and can contain colons.
But serde doesn't know, when reading, if a string is logically a "value" or a "key" in a map.
It means that if we allow colons in quoteles strings the following Hjson

	{
		key: value
	}

would be correctly interpreted when deserialized into a struct (because `key` is then known as an identifier) but wouldn't be correctly deserialized into `HashMap<String, String>`.

It seems to me the less surprising choice is to not allow colons in quoteless strings (they're hard to parse for an human too anyway) until I find how to reliably parse quoteless map keys with Serde 1.0.
