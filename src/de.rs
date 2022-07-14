use crate::read::{self, Read};
use crate::{Error, Result};

pub struct Deserializer<R> {
    read: R,
}

impl<'de, R: Read<'de>> Deserializer<R> {
    fn new(read: R) -> Self {
        Self { read }
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        self.read.peek()
    }

    fn eat_char(&mut self) {
        self.read.discard();
    }

    fn next_char(&mut self) -> Result<Option<u8>> {
        self.read.next()
    }

    fn end(&mut self) -> Result<()> {
        match self.peek()? {
            Some(_) => Err(Error {}),
            None => Ok(()),
        }
    }
}

impl<'de, 'a, R: Read<'de>> serde::de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek()?.ok_or(Error {})?;
        match peek {
            b'!' => {
                self.eat_char();
                let peek = self.peek()?.ok_or(Error {})?;
                match peek {
                    b'n' => {
                        self.eat_char();
                        visitor.visit_unit()
                    }
                    b't' => {
                        self.eat_char();
                        visitor.visit_bool(true)
                    }
                    b'f' => {
                        self.eat_char();
                        visitor.visit_bool(false)
                    }
                    b'(' => {
                        self.eat_char();

                        let ret = visitor.visit_seq(SeqAccess::new(self));
                        let end_seq = if let Some(b')') = self.peek()? {
                            self.eat_char();
                            Ok(())
                        } else {
                            Err(Error {})
                        };

                        match (ret, end_seq) {
                            (Ok(ret), Ok(())) => Ok(ret),
                            _ => Err(Error {}),
                        }
                    }
                    _ => Err(Error {}),
                }
            }
            b'-' | b'0'..=b'9' => todo!(),
            b'\'' => {
                self.eat_char();
                let s = self.read.parse_str()?;

                visitor.visit_string(s)
            }
            b'(' => {
                self.eat_char();

                let ret = visitor.visit_map(MapAccess::new(self));

                let end_map = if let Some(b')') = self.peek()? {
                    self.eat_char();
                    Ok(())
                } else {
                    Err(Error {})
                };

                match (ret, end_map) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    _ => Err(Error {}),
                }
            }
            _ => {
                let value = self.read.parse_ident()?;

                visitor.visit_string(value)
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek()?.ok_or(Error {})?;

        let value = if peek == b'!' {
            self.eat_char();
            let next = self.next_char()?.ok_or(Error {})?;
            match next {
                b't' => true,
                b'f' => false,
                _ => return Err(Error {}),
            }
        } else {
            return Err(Error {});
        };

        visitor.visit_bool(value)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek()?.ok_or(Error {})?;

        let value = match peek {
            b'!' => return Err(Error {}),
            b'\'' => {
                self.eat_char();
                self.read.parse_str()?
            }
            b'-' | b'0'..=b'9' => todo!(),
            _ => self.read.parse_ident()?,
        };

        visitor.visit_string(value)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.peek()? {
            Some(b'!') => {
                self.eat_char();
                if self.next_char()? != Some(b'n') {
                    return Err(Error {});
                }
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek()?.ok_or(Error {})?;
        match peek {
            b'!' => {
                self.eat_char();
                let peek = self.peek()?.ok_or(Error {})?;
                match peek {
                    b'n' => {
                        self.eat_char();
                        visitor.visit_unit()
                    }
                    _ => Err(Error {}),
                }
            }
            _ => Err(Error {}),
        }
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek()?.ok_or(Error {})?;

        let value = match peek {
            b'!' => {
                self.eat_char();

                let peek = self.peek()?.ok_or(Error {})?;
                if peek != b'(' {
                    return Err(Error {});
                }

                self.eat_char();

                let ret = visitor.visit_seq(SeqAccess::new(self));

                let end_seq = if let Some(b')') = self.peek()? {
                    self.eat_char();
                    Ok(())
                } else {
                    Err(Error {})
                };

                match (ret, end_seq) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    _ => Err(Error {}),
                }
            }
            _ => return Err(Error {}),
        };

        value
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek()?.ok_or(Error {})?;

        let value = match peek {
            b'(' => {
                self.eat_char();

                let ret = visitor.visit_map(MapAccess::new(self));

                let end_map = if let Some(b')') = self.peek()? {
                    self.eat_char();
                    Ok(())
                } else {
                    Err(Error {})
                };

                match (ret, end_map) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    _ => Err(Error {}),
                }
            }
            _ => return Err(Error {}),
        };

        value
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek()?.ok_or(Error {})?;

        let value = match peek {
            b'(' => {
                self.eat_char();

                let ret = visitor.visit_map(MapAccess::new(self));

                let end_map = if let Some(b')') = self.peek()? {
                    self.eat_char();
                    Ok(())
                } else {
                    Err(Error {})
                };

                match (ret, end_map) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    _ => Err(Error {}),
                }
            }
            b'!' => todo!(),
            _ => return Err(Error {}),
        };

        value
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }
}

struct MapAccess<'d, R: 'd> {
    de: &'d mut Deserializer<R>,
    first: bool,
}

impl<'a, R: 'a> MapAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        MapAccess { de, first: true }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> serde::de::MapAccess<'de> for MapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        let peek = match self.de.peek()?.ok_or(Error {})? {
            b')' => return Ok(None),
            b',' if !self.first => {
                self.de.eat_char();
                self.de.peek()?.ok_or(Error {})?
            }
            b => {
                if self.first {
                    self.first = false;
                    b
                } else {
                    return Err(Error {});
                }
            }
        };

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.de.peek()?.ok_or(Error {})? {
            b':' => {
                self.de.eat_char();
            }
            _ => return Err(Error {}),
        }
        seed.deserialize(&mut *self.de)
    }
}

struct SeqAccess<'d, R: 'd> {
    de: &'d mut Deserializer<R>,
    first: bool,
}

impl<'a, R: 'a> SeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        SeqAccess { de, first: true }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> serde::de::SeqAccess<'de> for SeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let _peek = match self.de.peek()?.ok_or(Error {})? {
            b')' => return Ok(None),
            b',' if !self.first => {
                self.de.eat_char();

                self.de.peek()?.ok_or(Error {})?
            }
            b => {
                if self.first {
                    self.first = false;
                    b
                } else {
                    return Err(Error {});
                }
            }
        };

        seed.deserialize(&mut *self.de).map(Some)
    }
}

fn from_trait<'de, R, T>(read: R) -> Result<T>
where
    R: Read<'de>,
    T: serde::de::Deserialize<'de>,
{
    let mut de = Deserializer::new(read);
    let value = serde::de::Deserialize::deserialize(&mut de)?;

    de.end()?;

    Ok(value)
}

pub fn from_slice<'a, T>(v: &'a [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    from_trait(read::SliceRead::new(v))
}

#[cfg(test)]
mod test {
    #[test]
    fn deserialize_true() {
        let v: bool = super::from_slice(b"!t").unwrap();

        assert!(v);
    }
    #[test]
    fn deserialize_false() {
        let v: bool = super::from_slice(b"!f").unwrap();

        assert!(!v);
    }
    #[test]
    fn fail_deserialize_bool_trailing() {
        let v: super::Result<bool> = super::from_slice(b"!ff");

        assert!(matches!(v, Err(_)));
    }
    #[test]
    fn deserialize_quoted_empty_string() {
        let v: String = super::from_slice(b"''").unwrap();

        assert_eq!(v, "");
    }
    #[test]
    fn deserialize_quoted_string() {
        let v: String = super::from_slice(b"'hello, rison'").unwrap();

        assert_eq!(v, "hello, rison");
    }
    #[test]
    fn deserialize_quoted_string_with_escapes() {
        let v: String = super::from_slice(b"'hello, !'rison!'!!'").unwrap();

        assert_eq!(v, "hello, 'rison'!");
    }
    #[test]
    fn deserialize_ident_string() {
        let v: String = super::from_slice(b"hellorison").unwrap();

        assert_eq!(v, "hellorison");
    }
    #[test]
    fn deserialize_none() {
        let v: Option<String> = super::from_slice(b"!n").unwrap();

        assert_eq!(v, None);
    }
    #[test]
    fn deserialize_some_ident_string() {
        let v: Option<String> = super::from_slice(b"hellorison").unwrap();

        assert_eq!(v, Some("hellorison".into()));
    }
    #[test]
    fn deserialize_empty_struct() {
        #[derive(serde::Deserialize)]
        struct Empty {}
        let v: Empty = super::from_slice(b"()").unwrap();

        // assert!(v.is_empty());
    }
    #[test]
    fn deserialize_struct() {
        #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
        struct Full {
            a: String,
            b: String,
        }
        let v: Full = super::from_slice(b"(a:hello,b:world)").unwrap();

        assert_eq!(
            v,
            Full {
                a: "hello".into(),
                b: "world".into()
            }
        );
    }
    #[test]
    fn deserialize_map() {
        let v: std::collections::HashMap<String, String> =
            super::from_slice(b"(a:hello,b:world)").unwrap();

        let expected = vec![("a".into(), "hello".into()), ("b".into(), "world".into())]
            .into_iter()
            .collect();
        assert_eq!(v, expected);
    }
    #[test]
    fn deserialize_tuple() {
        let v: (String, String) = super::from_slice(b"!(hello,world)").unwrap();

        assert_eq!(v, ("hello".into(), "world".into()));
    }
    #[test]
    fn deserialize_value_string() {
        let v: serde_json::Value = super::from_slice(b"helloworld").unwrap();

        assert_eq!(v, serde_json::Value::String("helloworld".into()));
    }
    #[test]
    fn deserialize_value_map() {
        let v: serde_json::Value = super::from_slice(b"(hello:!(a,b,c),world:'it works')").unwrap();

        assert_eq!(
            v,
            serde_json::json!({"hello": ["a", "b", "c"], "world": "it works"})
        );
    }
}
