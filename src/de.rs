//! A Hjson deserializer.
//!
//! Inspiration and structure of this code comes in part from Serde's
//! tutorial at https://serde.rs/impl-deserializer.html
//!
use {
    crate::{
        de_enum::*,
        de_map::*,
        de_seq::*,
        error::{Error, Result},
    },
    serde::{
        Deserialize,
        de::{
            self,
            IntoDeserializer,
            Visitor,
        },
    },
    unescape::unescape,
};

pub struct Deserializer<'de> {
    input: &'de str, // what remains to be parsed

    // Make it possible to avoid reading a string as a quoteless
    // string when a key map is waited for (for example in
    //     {
    //         key: value
    //     }
    // so that the key doesn't go til the end of the line.
    pub(crate) accept_quoteless: bool,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer {
            input,
            accept_quoteless: true,
        }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    deserializer.eat_shit().ok();
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    /// Look at the first character in the input without consuming it.
    pub(crate) fn peek_char(&self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    /// remove the next character (which is assumed to be ch)
    pub(crate) fn drop(&mut self, ch: char) {
        self.input = &self.input[ch.len_utf8()..];
    }

    /// tells whether the next tree bytes are `'''` which
    /// is the start or end of a multiline string literal in Hjson
    pub(crate) fn is_at_triple_quote(&self, offset: usize) -> bool {
        self.input.len() >= offset + 3 && &self.input[offset..offset+3] == "'''"
    }

    pub(crate) fn eat_line(&mut self) -> Result<()> {
        self.accept_quoteless = true;
        match self.input.find('\n') {
            Some(len) => {
                self.input = &self.input[len + 1..];
                Ok(())
            }
            None => Err(Error::Eof),
        }
    }

    pub(crate) fn eat_until_star_slash(&mut self) -> Result<()> {
        match self.input.find("*/") {
            Some(len) => {
                self.input = &self.input[len + 2..];
                Ok(())
            }
            None => Err(Error::Eof),
        }
    }

    /// advance until the first non space character and
    /// return the number of eaten characters in the last
    /// line
    pub(crate) fn eat_spaces(&mut self) -> Result<usize> {
        let mut eaten_chars = 0;
        loop {
            let ch = self.peek_char()?;
            if ch == '\n' {
                self.accept_quoteless = true;
                self.drop(ch);
                eaten_chars = 0;
            } else if ch.is_whitespace() {
                self.drop(ch);
                eaten_chars += 1
            } else {
                return Ok(eaten_chars);
            }
        }
    }

    pub(crate) fn eat_shit(&mut self) -> Result<()> {
        self.eat_shit_and(None)
    }

    pub(crate) fn eat_shit_and(
        &mut self,
        mut including: Option<char>,
    ) -> Result<()> {
        let mut last_is_slash = false;
        loop {
            let ch = self.peek_char()?;
            match ch {
                '#' => {
                    self.eat_line()?;
                    last_is_slash = false;
                }
                '*' => {
                    if last_is_slash {
                        self.eat_until_star_slash()?;
                    } else {
                        self.drop(ch);
                    }
                    last_is_slash = false;
                }
                '/' => {
                    if last_is_slash {
                        self.eat_line()?;
                        last_is_slash = false;
                    } else {
                        self.drop(ch);
                        last_is_slash = true;
                    }
                }
                '\n' => {
                    self.accept_quoteless = true;
                    self.drop(ch);
                    last_is_slash = false;
                }
                _ if including == Some(ch) => {
                    self.drop(ch);
                    including = None;
                    last_is_slash = false;
                }
                _ if ch.is_whitespace() => {
                    self.drop(ch);
                    last_is_slash = false;
                }
                _ => {
                    return Ok(());
                }
            }
        }
    }

    /// Consume the first character in the input.
    pub(crate) fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    /// Parse the JSON identifier `true` or `false`.
    fn parse_bool(&mut self) -> Result<bool> {
        if self.input.starts_with("true") {
            self.input = &self.input["true".len()..];
            Ok(true)
        } else if self.input.starts_with("false") {
            self.input = &self.input["false".len()..];
            Ok(false)
        } else {
            Err(Error::ExpectedBoolean)
        }
    }

    /// read the characters of the coming integer, without parsing the
    /// resulting string
    fn read_integer(&mut self, unsigned: bool) -> Result<&'de str> {
        self.eat_shit()?;
        for (idx, ch) in self.input.char_indices() {
            match ch {
                '-' if unsigned => {
                    return Err(Error::ExpectedPositiveInteger);
                }
                '-' if idx > 0 => {
                    return Err(Error::Syntax);
                }
                '0'..='9' | '-' => {
                    // if it's too long, this will be handled at conversion
                }
                _ => {
                    let s = &self.input[..idx];
                    self.input = &self.input[idx..];
                    return Ok(s);
                }
            }
        }
        Err(Error::Eof)
    }

    /// read the characters of the coming floating point number, without parsing
    fn read_float(&mut self) -> Result<&'de str> {
        self.eat_shit()?;
        for (idx, ch) in self.input.char_indices() {
            match ch {
                '0'..='9' | '-' | '+' | '.' | 'e' | 'E' => {
                    // if it's invalid, this will be handled at conversion
                }
                _ => {
                    let s = &self.input[..idx];
                    self.input = &self.input[idx..];
                    return Ok(s);
                }
            }
        }
        Err(Error::Eof)
    }

    /// Parse a string until the next unescaped quote.
    //  This function doesn't manage escaping (I don't
    //  know how to deal with building a string with
    //  serde's lifetime system)
    fn parse_quoted_str(&mut self) -> Result<&'de str> {
        if self.next_char()? != '"' {
            // should not happen
            return Err(Error::ExpectedString);
        }
        match self.input.find('"') {
            Some(len) => {
                let s = &self.input[..len];
                self.input = &self.input[len + 1..];
                Ok(s)
            }
            None => Err(Error::Eof),
        }
    }


    /// Parse a string until the next unescaped quote
    fn parse_quoted_string(&mut self) -> Result<String> {
        match unescape(self.parse_quoted_str()?) {
            Some(s) => Ok(s),
            None => Err(Error::InvalidEscapeSequence),
        }
    }

    /// Parse a string until end of line or colon.
    fn parse_quoteless_str(&mut self) -> Result<&'de str> {
        self.eat_shit()?;
        for (idx, ch) in self.input.char_indices() {
            match ch {
                '\n' => {
                    let s = &self.input[..idx];
                    self.input = &self.input[idx + 1..];
                    return Ok(s);
                }
                _ => {}
            }
        }
        Err(Error::Eof)
    }

    /// Parse a string until the next triple quote.
    ///
    fn parse_multiline_string(&mut self) -> Result<String> {
        if !self.is_at_triple_quote(0) {
            // We could probably assume the first three bytes
            // are "'''" and can be dropped without check
            return Err(Error::ExpectedString);
        }
        self.input = &self.input[3..];
        self.eat_line()?;
        // we count the spaces on the first line
        let indent = self.eat_spaces()?;
        let mut v = String::new();
        let mut line_len = indent;
        for (idx, ch) in self.input.char_indices() {
            match ch {
                '\'' if self.is_at_triple_quote(idx) => {
                    self.input = &self.input[idx+3..];
                    v.truncate(v.trim_end().len()); // is there faster ?
                    return Ok(v);
                }
                '\n' => {
                    v.push(ch);
                    line_len = 0;
                }
                _ => {
                    if line_len >= indent || !ch.is_whitespace() {
                        v.push(ch);
                    }
                    line_len += 1;
                }
            }
        }
        Err(Error::Eof)
    }

    /// parse a map key without quotes
    fn parse_quoteless_identifier(&mut self) -> Result<&'de str> {
        self.eat_shit()?;
        for (idx, ch) in self.input.char_indices() {
            match ch {
                '"' | ',' | '[' | ']' | '{' | '}' => {
                    return Err(Error::Syntax);
                }
                ' ' => {
                    let s = &self.input[..idx];
                    self.input = &self.input[idx + 1..];
                    return Ok(s);
                }
                ':' => {
                    let s = &self.input[..idx];
                    self.input = &self.input[idx..]; // we keep the colon
                    return Ok(s);
                }
                _ => {}
            }
        }
        Err(Error::Eof)
    }

    /// parse a string which may be a value
    /// (i.e. not an map key or variant identifier ).
    /// This function returns a borrowed string, which means it
    /// can't manage multiline strings (which don't map to a
    /// part of the source).
    fn parse_str_value(&mut self) -> Result<&'de str> {
        self.eat_shit()?;
        let ch = self.peek_char()?;
        match ch {
            ',' | ':' | '[' | ']' | '{' | '}' => Err(Error::Syntax),
            '"' => self.parse_quoted_str(),
            _ => self.parse_quoteless_str(),
        }
    }

    /// parse a string which may be a value
    /// (i.e. not an map key or variant identifier )
    fn parse_string_value(&mut self) -> Result<String> {
        self.eat_shit()?;
        let ch = self.peek_char()?;
        match ch {
            ',' | ':' | '[' | ']' | '{' | '}' => Err(Error::Syntax),
            '\'' if self.is_at_triple_quote(0) => self.parse_multiline_string(),
            '"' => self.parse_quoted_string(),
            _ => (
                if self.accept_quoteless {
                    self.parse_quoteless_str()
                } else {
                    self.parse_quoteless_identifier()
                }
            ).map(|s| s.to_string()),
        }
    }

    fn parse_identifier(&mut self) -> Result<&'de str> {
        self.eat_shit()?;
        let ch = self.peek_char()?;
        // we set accept_quoteless to true so that a quoteless
        // string can be accepted *after* the current identifier
        self.accept_quoteless = true;
        match ch {
            ',' | ':' | '[' | ']' | '{' | '}' => Err(Error::Syntax),
            '"' => self.parse_quoted_str(),
            _ => self.parse_quoteless_identifier(),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.eat_shit()?;
        // TODO look ahead to decide between integers and floats
        // (and maybe size)
        match self.peek_char()? {
            'n' => self.deserialize_unit(visitor),
            't' | 'f' => self.deserialize_bool(visitor),
            '"' => self.deserialize_str(visitor),
            '0'..='9' => self.deserialize_f64(visitor),
            '-' => self.deserialize_f64(visitor),
            '[' => self.deserialize_seq(visitor),
            '{' => self.deserialize_map(visitor),
            _ => Err(Error::Syntax),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_integer(false)
            .and_then(|s| {
                s.parse().map_err(|_| Error::Message(format!("not a valid i8")))
            })?;
        visitor.visit_i8(v)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_integer(false)
            .and_then(|s| {
                s.parse().map_err(|_| Error::Message(format!("not a valid i16")))
            })?;
        visitor.visit_i16(v)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_integer(false)
            .and_then(|s| {
                s.parse().map_err(|_| Error::Message(format!("not a valid i32")))
            })?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_integer(false)
            .and_then(|s| {
                s.parse().map_err(|_| Error::Message(format!("not a valid i64")))
            })?;
        visitor.visit_i64(v)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_integer(true)
            .and_then(|s| {
                s.parse().map_err(|_| Error::Message(format!("not a valid u8")))
            })?;
        visitor.visit_u8(v)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_integer(true)
            .and_then(|s| {
                s.parse().map_err(|_| Error::Message(format!("not a valid u16")))
            })?;
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_integer(true)
            .and_then(|s| {
                s.parse().map_err(|_| Error::Message(format!("not a valid u32")))
            })?;
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_integer(true)
            .and_then(|s| {
                s.parse().map_err(|_| Error::Message(format!("not a valid u64")))
            })?;
        visitor.visit_u64(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_float()
            .and_then(|s| {
                s.parse().map_err(|_| Error::Message(format!("not a valid f32")))
            })?;
        visitor.visit_f32(v)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.read_float()
            .and_then(|s| {
                s.parse().map_err(|_| Error::Message(format!("not a valid f64")))
            })?;
        visitor.visit_f64(v)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let c = self.parse_string_value()
            .and_then(|s| {
                s.chars().next()
                    .ok_or_else(|| Error::ExpectedSingleChar)
            })?;
        visitor.visit_char(c)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_str_value()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.parse_string_value()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.eat_shit()?;
        if self.input.starts_with("null") {
            self.input = &self.input["null".len()..];
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.eat_shit()?;
        if self.input.starts_with("null") {
            self.input = &self.input["null".len()..];
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNull)
        }
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.eat_shit()?;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.eat_shit()?;
        if self.next_char()? == '[' {
            let value = visitor.visit_seq(SeqReader::new(&mut self))?;
            if self.next_char()? == ']' {
                Ok(value)
            } else {
                Err(Error::ExpectedArrayEnd)
            }
        } else {
            Err(Error::ExpectedArray)
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.eat_shit()?;
        if self.next_char()? == '{' {
            let value = visitor.visit_map(MapReader::new(&mut self))?;
            self.eat_shit()?;
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedMap)
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.eat_shit()?;
        if self.peek_char()? == '"' {
            // Visit a unit variant.
            visitor.visit_enum(self.parse_quoted_str()?.into_deserializer())
        } else if self.next_char()? == '{' {
            // Visit a newtype variant, tuple variant, or struct variant.
            let value = visitor.visit_enum(EnumReader::new(self))?;
            self.eat_shit()?;
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedEnum)
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_identifier()?)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
