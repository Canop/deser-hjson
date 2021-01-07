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

Hjson is a good language for a configuration file.
Such files should be written by a human, read and modified by other humans, then deserialized into a precise structure by a program:

```rust
let file_content = fs::read_to_string(&file_path)?;
let configuration = deser_hjson::from_str(&file_content);
```

If the configuration file is invalid or doesn't match the expected type, the error details the expectation and the error precise location.

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

## Performances

A serious benchmark is missing now but this deserializer seems fast enough.

[Broot](https://dystroy.org/broot) can be configured either with TOML or with Hjson (the selection is dynamic, based on the file extension).

Broot configuration loading takes about 100µs when in Hjson while it takes about 2ms in TOML (so Hjson seems to be 20 times faster than TOML on this simple almost anecdotical test).
It's very possible the TOML deserializer is slower only because it's more strict and does more checks, though (I'm not familiar enough with its implementation).
