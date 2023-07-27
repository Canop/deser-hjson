use {
    crate::{
        de::Deserializer,
        error::{Error, Result},
    },
    serde::de::{DeserializeSeed, SeqAccess},
};

/// an implementation of serde's SeqAccess interface which
/// is used to deserialize arrays
pub struct SeqReader<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> SeqReader<'a, 'de> {
    pub fn new(de: &'a mut Deserializer<'de>) -> Self {
        SeqReader { de }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for SeqReader<'a, 'de> {
    type Error = Error;

    /// read an array item and eat the optional comma which may follow it
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        self.de.eat_shit()?;
        if self.de.peek_char()? == ']' {
            return Ok(None);
        }
        let v = seed.deserialize(&mut *self.de)?;
        self.de.eat_shit_and(Some(','))?;
        Ok(Some(v))
    }
}
