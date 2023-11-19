use {
    crate::{
        de::Deserializer,
        error::{Error, ErrorCode::*, Result},
    },
    serde::de::{DeserializeSeed, MapAccess},
};

pub struct MapReader<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    /// if braceless is true, the map may be closed by an eof instead of a '}'
    pub braceless: bool,
}

impl<'a, 'de> MapReader<'a, 'de> {
    pub fn braceless(de: &'a mut Deserializer<'de>) -> Self {
        MapReader { de, braceless: true }
    }
    pub fn within_braces(de: &'a mut Deserializer<'de>) -> Self {
        MapReader { de, braceless: false }
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for MapReader<'a, 'de> {
    type Error = Error;

    /// read a map key and the following colon
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if let Err(e) = self.de.eat_shit_and(Some(',')) {
            if !self.braceless || !e.is_eof() {
                return Err(e);
            }
        }
        match self.de.peek_byte() {
            Ok(b'}') => { return Ok(None); }
            Err(e) => {
                if e.is_eof() && self.braceless {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
            _ => {}
        }
        // Here's there's a problem: if the key is a string it should be
        // parsed as an identifier but serde will call deserialize_string.
        // The problem here is that I thus can't accept colons in quoteless
        // strings, even when not in a identifier location :\
        self.de.accept_quoteless_value = false;
        let v = seed.deserialize(&mut *self.de)?;
        self.de.eat_shit()?;
        if self.de.next_byte()? == b':' {
            Ok(Some(v))
        } else {
            self.de.fail(ExpectedMapColon)
        }
    }

    /// read a map value and eat the optional comma which may follow it
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        self.de.eat_shit()?;
        match seed.deserialize(&mut *self.de) {
            Err(e) => self.de.cook_err(e),
            Ok(v) => {
                if let Err(e) = self.de.eat_shit_and(Some(',')) {
                    if !self.braceless || !e.is_eof() {
                        return Err(e);
                    }
                }
                Ok(v)
            }
        }
    }
}
