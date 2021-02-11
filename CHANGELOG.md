
<a name="v0.1.11"></a>
### v0.1.11 - 2021-02-11
- fix primitive types (ie not Hjson texts but primitives like integers and floats) needing a space at the end - Fix #1

<a name="v0.1.10"></a>
### v0.1.10 - 2021-02-11
- make from_str parse a `DeserializeOwned` instead of a borrowed `Deserialize<'a>`
