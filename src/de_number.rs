use {
    crate::{
        de::Deserializer,
        error::{ErrorCode::*, Result},
    },
    serde::de::Visitor,
};

/// an intermediate representation of number which
/// are read into undefinite types
pub(crate) struct Number<'de> {
    negative: bool,
    s: &'de str,
    has_float_chars: bool,
}

impl<'de> Number<'de> {
    /// read the characters of the coming floating point number, without parsing.
    /// The sign at the start is assumed to have been already read
    pub fn read<'a>(
        de: &'a mut Deserializer<'de>,
    ) -> Result<Self> {
        de.eat_shit()?;
        let mut negative = false;
        let mut has_float_chars = false;
        for (idx, ch) in de.input().char_indices() {
            match ch {
                '0'..='9' => { }
                '-' if idx == 0 => {
                    negative = true;
                }
                '-' | '+' | '.' | 'e' | 'E' => {
                    has_float_chars = true;
                }
                _ => {
                    let s = de.start(idx);
                    de.advance(idx); // we keep the last char
                    return Ok(Self {
                        negative, s, has_float_chars
                    });
                }
            }
        }
        let s = de.take_all();
        Ok(Self {
            negative, s, has_float_chars
        })
    }
    /// deserialize into a relevant number type
    pub fn visit<'a, V>(
        &self,
        de: &'a mut Deserializer<'de>,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.has_float_chars {
            // this is a floating point number (or an error)
            let v: f64 = self.s.parse()
                .map_err(|_| de.err(ExpectedF64))?;
            visitor.visit_f64(v)
        } else if self.negative {
            // this is a negative integer (or an error)
            let v: i64 = self.s.parse()
                .map_err(|_| de.err(ExpectedI64))?;
            visitor.visit_i64(v)
        } else {
            // this is a positive integer (or a number)
            let v: u64 = self.s.parse()
                .map_err(|_| de.err(ExpectedU64))?;
            visitor.visit_u64(v)
        }
    }
}
