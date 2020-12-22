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
        error::{
            Error,
            ErrorCode::{self, *},
            Result,
        },
    },
    serde::de::{self, IntoDeserializer, Visitor},
    unescape::unescape,
};

/// The deserializer. You normally don't call it directly
/// but use the `from_str` function available at crate's level.
pub struct Deserializer<'de> {
    // the complete string we received
    src: &'de str,

    // where we're at
    pos: usize,

    // Make it possible to avoid reading a string as a quoteless
    // string when a key map is waited for (for example in
    //     {
    //         key: value
    //     }
    // ) so that the key doesn't go til the end of the line.
    pub(crate) accept_quoteless: bool,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(src: &'de str) -> Self {
        Deserializer {
            src,
            pos: 0,
            accept_quoteless: true,
        }
    }

    pub(crate) fn err(&self, code: ErrorCode) -> Error {
        // we compute the number of lines and columns to current pos
        let (mut line, mut col) = (1, 1);
        for ch in self.src[..self.pos].chars() {
            if ch == '\n' {
                col = 1;
                line += 1;
            } else {
                col += 1;
            }
        }
        let at = self.input().chars().take(15).collect();
        Error::Syntax {
            line,
            col,
            code,
            at,
        }
    }

    pub(crate) fn fail<T>(&self, code: ErrorCode) -> Result<T> {
        Err(self.err(code))
    }

    /// return an error if there's more than just spaces
    /// and comments in the remaining input
    pub fn check_all_consumed(&mut self) -> Result<()> {
        self.eat_shit().ok();
        if self.input().is_empty() {
            Ok(())
        } else {
            self.fail(TrailingCharacters)
        }
    }

    /// what remains to be parsed (including the
    /// character we peeked at, if any)
    fn input(&self) -> &'de str {
        &self.src[self.pos..]
    }

    /// Look at the first character in the input without consuming it.
    pub(crate) fn peek_char(&self) -> Result<char> {
        match self.input().chars().next() {
            Some(ch) => Ok(ch),
            _ => self.fail(Eof),
        }
    }

    /// return the `len` first bytes of the input, without checking anything
    /// (assuming it has been done) nor consuming anything
    pub(crate) fn start(&self, len: usize) -> &'de str {
        &self.src[self.pos..self.pos + len]
    }

    /// remove the next character (which is assumed to be ch)
    pub(crate) fn drop(&mut self, ch: char) {
        self.advance(ch.len_utf8());
    }

    /// remove the next character (which is assumed to be ch)
    pub(crate) fn advance(&mut self, bytes_count: usize) {
        self.pos += bytes_count;
    }

    /// Consume the first character in the input.
    pub(crate) fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.drop(ch);
        Ok(ch)
    }

    /// tells whether the next tree bytes are `'''` which
    /// is the start or end of a multiline string literal in Hjson
    pub(crate) fn is_at_triple_quote(&self, offset: usize) -> bool {
        self.src.len() >= self.pos + offset + 3
            && &self.src[offset + self.pos..offset + self.pos + 3] == "'''"
    }

    pub(crate) fn eat_line(&mut self) -> Result<()> {
        self.accept_quoteless = true;
        match self.input().find('\n') {
            Some(len) => {
                self.advance(len + 1);
                Ok(())
            }
            None => self.fail(Eof),
        }
    }

    pub(crate) fn eat_until_star_slash(&mut self) -> Result<()> {
        match self.input().find("*/") {
            Some(len) => {
                self.advance(len + 2);
                Ok(())
            }
            None => self.fail(Eof),
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

    pub(crate) fn eat_shit_and(&mut self, mut including: Option<char>) -> Result<()> {
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

    /// Parse the JSON identifier `true` or `false`.
    fn parse_bool(&mut self) -> Result<bool> {
        if self.input().starts_with("true") {
            self.advance("true".len());
            Ok(true)
        } else if self.input().starts_with("false") {
            self.advance("false".len());
            Ok(false)
        } else {
            self.fail(ExpectedBoolean)
        }
    }

    /// read the characters of the coming integer, without parsing the
    /// resulting string
    fn read_integer(&mut self, unsigned: bool) -> Result<&'de str> {
        self.eat_shit()?;
        for (idx, ch) in self.input().char_indices() {
            match ch {
                '-' if unsigned => {
                    return self.fail(ExpectedPositiveInteger);
                }
                '-' if idx > 0 => {
                    return self.fail(UnexpectedChar);
                }
                '0'..='9' | '-' => {
                    // if it's too long, this will be handled at conversion
                }
                _ => {
                    let s = self.start(idx);
                    self.advance(idx); // we keep the last char
                    return Ok(s);
                }
            }
        }
        self.fail(Eof)
    }

    /// read the characters of the coming floating point number, without parsing
    fn read_float(&mut self) -> Result<&'de str> {
        self.eat_shit()?;
        for (idx, ch) in self.input().char_indices() {
            match ch {
                '0'..='9' | '-' | '+' | '.' | 'e' | 'E' => {
                    // if it's invalid, this will be handled at conversion
                }
                _ => {
                    let s = self.start(idx);
                    self.advance(idx); // we keep the last char
                    return Ok(s);
                }
            }
        }
        self.fail(Eof)
    }

    /// Parse a string until the next unescaped quote.
    //  This function doesn't manage escaping (I don't
    //  know how to deal with building a string with
    //  serde's lifetime system)
    fn parse_quoted_str(&mut self) -> Result<&'de str> {
        if self.next_char()? != '"' {
            // should not happen
            return self.fail(ExpectedString);
        }
        match self.input().find('"') {
            Some(len) => {
                let s = self.start(len);
                self.advance(len + 1); // we consume the '"'
                Ok(s)
            }
            None => self.fail(Eof),
        }
    }

    /// Parse a string until the next unescaped quote
    fn parse_quoted_string(&mut self) -> Result<String> {
        match unescape(self.parse_quoted_str()?) {
            Some(s) => Ok(s),
            None => self.fail(InvalidEscapeSequence),
        }
    }

    /// Parse a string until end of line
    fn parse_quoteless_str(&mut self) -> Result<&'de str> {
        self.eat_shit()?;
        for (idx, ch) in self.input().char_indices() {
            match ch {
                '\n' => {
                    let s = self.start(idx);
                    self.advance(idx + 1);
                    return Ok(s);
                }
                _ => {}
            }
        }
        self.fail(Eof)
    }

    /// Parse a string until the next triple quote.
    fn parse_multiline_string(&mut self) -> Result<String> {
        if !self.is_at_triple_quote(0) {
            // We could probably assume the first three bytes
            // are "'''" and can be dropped without check
            return self.fail(ExpectedString);
        }
        self.advance(3);
        self.eat_line()?;
        // we count the spaces on the first line
        let indent = self.eat_spaces()?;
        let mut v = String::new();
        let mut line_len = indent;
        for (idx, ch) in self.input().char_indices() {
            match ch {
                '\'' if self.is_at_triple_quote(idx) => {
                    self.advance(idx + 3);
                    v.truncate(v.trim_end().len()); // trimming end
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
        self.fail(Eof)
    }

    /// parse a map key without quotes
    fn parse_quoteless_identifier(&mut self) -> Result<&'de str> {
        self.eat_shit()?;
        for (idx, ch) in self.input().char_indices() {
            match ch {
                '"' | ',' | '[' | ']' | '{' | '}' => {
                    return self.fail(UnexpectedChar);
                }
                ' ' => {
                    let s = self.start(idx);
                    self.advance(idx + 1);
                    return Ok(s);
                }
                ':' => {
                    let s = self.start(idx);
                    self.advance(idx); // we keep the colon
                    return Ok(s);
                }
                _ => {}
            }
        }
        self.fail(Eof)
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
            ',' | ':' | '[' | ']' | '{' | '}' => self.fail(UnexpectedChar),
            '"' => self.parse_quoted_str(),
            _ => self.parse_quoteless_str(),
        }
    }

    /// parse a string which may be a value
    /// (i.e. not an map key or variant identifier )
    fn parse_string_value(&mut self) -> Result<String> {
        self.eat_shit()?;
        let ch = self.peek_char()?;
        let v = match ch {
            ',' | ':' | '[' | ']' | '{' | '}' => self.fail(UnexpectedChar),
            '\'' if self.is_at_triple_quote(0) => self.parse_multiline_string(),
            '"' => self.parse_quoted_string(),
            _ => (if self.accept_quoteless {
                self.parse_quoteless_str()
            } else {
                self.parse_quoteless_identifier()
            })
            .map(|s| s.to_string()),
        };
        self.accept_quoteless = true;
        v
    }

    fn parse_identifier(&mut self) -> Result<&'de str> {
        self.eat_shit()?;
        let ch = self.peek_char()?;
        // we set accept_quoteless to true so that a quoteless
        // string can be accepted *after* the current identifier
        self.accept_quoteless = true;
        match ch {
            ',' | ':' | '[' | ']' | '{' | '}' => self.fail(UnexpectedChar),
            '"' => self.parse_quoted_str(),
            _ => self.parse_quoteless_identifier(),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.eat_shit()?;
        // TODO look ahead to decide between integers and floats (and maybe size)
        match self.peek_char()? {
            '"' => self.deserialize_str(visitor),
            '0'..='9' => self.deserialize_f64(visitor),
            '-' => self.deserialize_f64(visitor),
            '[' => self.deserialize_seq(visitor),
            '{' => self.deserialize_map(visitor),
            _ => {
                let s = self.parse_str_value()?;
                match s {
                    "null" => visitor.visit_unit(),
                    "false" => visitor.visit_bool(false),
                    "true" => visitor.visit_bool(true),
                    _ => visitor.visit_borrowed_str(s),
                }
            }
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
        let v = self
            .read_integer(false)
            .and_then(|s| s.parse().map_err(|_| self.err(ExpectedI8)))?;
        visitor.visit_i8(v)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self
            .read_integer(false)
            .and_then(|s| s.parse().map_err(|_| self.err(ExpectedI16)))?;
        visitor.visit_i16(v)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self
            .read_integer(false)
            .and_then(|s| s.parse().map_err(|_| self.err(ExpectedI32)))?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self
            .read_integer(false)
            .and_then(|s| s.parse().map_err(|_| self.err(ExpectedI64)))?;
        visitor.visit_i64(v)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self
            .read_integer(true)
            .and_then(|s| s.parse().map_err(|_| self.err(ExpectedU8)))?;
        visitor.visit_u8(v)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self
            .read_integer(true)
            .and_then(|s| s.parse().map_err(|_| self.err(ExpectedU16)))?;
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self
            .read_integer(true)
            .and_then(|s| s.parse().map_err(|_| self.err(ExpectedU32)))?;
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self
            .read_integer(true)
            .and_then(|s| s.parse().map_err(|_| self.err(ExpectedU64)))?;
        visitor.visit_u64(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self
            .read_float()
            .and_then(|s| s.parse().map_err(|_| self.err(ExpectedF32)))?;
        visitor.visit_f32(v)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self
            .read_float()
            .and_then(|s| s.parse().map_err(|_| self.err(ExpectedF64)))?;
        visitor.visit_f64(v)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let c = self
            .parse_string_value()
            .and_then(|s| s.chars().next().ok_or_else(|| self.err(ExpectedSingleChar)))?;
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
        if self.input().starts_with("null") {
            self.advance("null".len());
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
        if self.input().starts_with("null") {
            self.advance("null".len());
            visitor.visit_unit()
        } else {
            self.fail(ExpectedNull)
        }
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
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
                self.fail(ExpectedArrayEnd)
            }
        } else {
            self.fail(ExpectedArray)
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
                self.fail(ExpectedMapEnd)
            }
        } else {
            self.fail(ExpectedMap)
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
                self.fail(ExpectedMapEnd)
            }
        } else {
            self.fail(ExpectedEnum)
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