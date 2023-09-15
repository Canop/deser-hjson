use {
    crate::{
        de::Deserializer,
        error::{Result},
    },
    serde::de::Visitor,
};

/// an intermediate representation of number which
/// are read into undefinite types, or a string if it fails
pub(crate) struct NumberOrString<'de> {
    s: &'de str,
}

impl<'de> NumberOrString<'de> {
    /// read the characters of the coming (maybe) number, without parsing
    pub fn read<'a>(
        de: &'a mut Deserializer<'de>,
    ) -> Result<Self> {
        for (idx, ch) in de.input().char_indices() {
            let stop = match ch {
                ',' | ':' | '{' | '}' | '[' | ']' => true,
                c if c.is_whitespace() => true,
                _ => false
            };
            if stop {
                let s = de.start(idx);
                de.advance(idx); // we keep the last char
                return Ok(Self {s});
            }
        }
        let s = de.take_all();
        Ok(Self {s})
    }
    /// deserialize into a relevant number type
    pub fn visit<'a, V>(
        &self,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'a>,
    {
        /* try by starting from the least general */
        if let Ok(v) = self.s.parse::<u64>() {
            visitor.visit_u64(v)
        }
        else if let Ok(v) = self.s.parse::<i64>() {
            visitor.visit_i64(v)
        }
        else if let Ok(v) = self.s.parse::<f64>() {
            visitor.visit_f64(v)
        }
        else {
            visitor.visit_string(self.s.to_string())
        }
    }
}
