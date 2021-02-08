use {
    crate::{
        de::Deserializer,
        error::{Error, ErrorCode::*, Result},
    },
    serde::de::{DeserializeSeed, MapAccess},
};

pub struct MapReader<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> MapReader<'a, 'de> {
    pub fn new(de: &'a mut Deserializer<'de>) -> Self {
        MapReader { de }
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
        self.de.eat_shit_and(Some(','))?;
        if self.de.peek_char()? == '}' {
            return Ok(None);
        }
        // Here's there's a problem: if the key is a string it should be
        // parsed as an identifier but serde will call deserialize_string.
        // The problem here is that I thus can't accept colons in quoteless
        // strings, even when not in a identifier location :\
        self.de.accept_quoteless_value = false;
        let v = seed.deserialize(&mut *self.de)?;
        self.de.eat_shit()?;
        if self.de.next_char()? == ':' {
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
                self.de.eat_shit_and(Some(','))?;
                Ok(v)
            }
        }
    }
}
