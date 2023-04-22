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

This is a Serde deserializer for [Hjson](https://hjson.github.io/), tailored for derive powered deserialization.

Hjson is a good language for a configuration file.
Such files should be written by a human, read and modified by other humans, then deserialized into a precise structure by a program:

```rust
let file_content = fs::read_to_string(&file_path)?;
let configuration = deser_hjson::from_str(&file_content);
```

If the configuration file is invalid or doesn't match the expected type, the error details the expectation and the error precise location.

## Example


```rust
use {
    deser_hjson::*,
    serde::Deserialize,
    std::collections::HashMap,
};
// This Hjson document comes from https://hjson.github.io/
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
// Here's the deserialization and the equality check:
assert_eq!(expected, from_str(hjson).unwrap());
```

## Known open-source usages

* [Broot](https://dystroy.org/broot) can be configured either with TOML or with Hjson (the selection is dynamic, based on the file extension).

* [lemmy](https://github.com/LemmyNet/lemmy) is configured in Hjson

* [Resc](https://github.com/Canop/resc) can be configured either with JSON or with Hjson

In all my tests, deserializing as Hjson was faster than JSON (even with a JSON file) and *much* faster than TOML.

## FAQ

### Does it work with JSON ?

Yes as any JSON file can be read as Hjson.

### Why only a derive-based deserializer?

Guessing the types in a format with implicit typing is way too dangereous.
When your user typed `false`, was it a string or a boolean ? When she typed `3`, was it as string or a number ?
While [not as crazy as YAML](https://hitchdev.com/strictyaml/why/implicit-typing-removed/), Hjson has no internal guard for this, and thus should only be deserialized into explicit types.

### Why a deserializer and no serializer?

Hjson isn't a data exchange format. It's intended to be written by humans, be full of comments and with a meaningful formatting.
While serializers would make sense in some context, they would have to be template based, or offer other means to specify comments and formatting, and serde isn't the right tool for that.
