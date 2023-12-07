use crate::error::{Error, ErrorKind, Result};
use crate::read::{self, Read};

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
            Some(_) => Err(Error {
                kind: ErrorKind::Syntax,
            }),
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
        match self.peek()? {
            Some(b'!') => {
                self.eat_char();
                let peek = self.peek()?.ok_or(Error {
                    kind: ErrorKind::Eof,
                })?;
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

                        if let b')' = self.peek()?.ok_or(Error {
                            kind: ErrorKind::Eof,
                        })? {
                            self.eat_char();
                        } else {
                            return Err(Error {
                                kind: ErrorKind::Syntax,
                            });
                        };

                        ret
                    }
                    _ => Err(Error {
                        kind: ErrorKind::Syntax,
                    }),
                }
            }
            Some(b'-' | b'0'..=b'9') => {
                let mut f = String::new();
                while let Some(ch @ (b'-' | b'0'..=b'9' | b'.')) = self.peek()? {
                    f.push(ch as char);
                    self.eat_char();
                }

                let v = f.parse().map_err(|e| Error {
                    kind: ErrorKind::Syntax,
                })?;

                visitor.visit_f64(v)
            }
            Some(b'\'') => {
                self.eat_char();
                let s = self.read.parse_str()?;

                visitor.visit_string(s)
            }
            Some(b'(') => {
                self.eat_char();

                let ret = visitor.visit_map(MapAccess::new(self));

                if let b')' = self.peek()?.ok_or(Error {
                    kind: ErrorKind::Eof,
                })? {
                    self.eat_char();
                } else {
                    return Err(Error {
                        kind: ErrorKind::Syntax,
                    });
                };

                ret
            }
            Some(_) => {
                let value = self.read.parse_ident()?;
                visitor.visit_string(value)
            }
            None => Err(Error {
                kind: ErrorKind::Eof,
            }),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.peek()? {
            Some(b'!') => {
                self.eat_char();
                if self.next_char()? != Some(b'n') {
                    return Err(Error {
                        kind: ErrorKind::Semantic,
                    });
                }
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
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
        match self.de.peek()? {
            Some(b')') => return Ok(None),
            Some(b',') if !self.first => {
                self.de.eat_char();
            }
            Some(_) => {
                if self.first {
                    self.first = false;
                } else {
                    return Err(Error {
                        kind: ErrorKind::Syntax,
                    });
                }
            }
            None => {
                return Err(Error {
                    kind: ErrorKind::Eof,
                });
            }
        };

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.de.peek()? {
            Some(b':') => {
                self.de.eat_char();
            }
            _ => {
                return Err(Error {
                    kind: ErrorKind::Syntax,
                })
            }
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
        match self.de.peek()? {
            Some(b')') => return Ok(None),
            Some(b',') if !self.first => {
                self.de.eat_char();
            }
            Some(_) => {
                if self.first {
                    self.first = false;
                } else {
                    return Err(Error {
                        kind: ErrorKind::Syntax,
                    });
                }
            }
            None => {
                return Err(Error {
                    kind: ErrorKind::Eof,
                })
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
pub fn from_str<'a, T>(v: &'a str) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    from_trait(read::StrRead::new(v))
}

pub fn from_reader<'a, T, I>(v: I) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
    I: std::io::Read,
{
    from_trait(read::IoRead::new(v))
}

#[cfg(test)]
mod test {
    #[test]
    fn deserialize_true() {
        let v: bool = super::from_str("!t").unwrap();

        assert!(v);
    }
    #[test]
    fn deserialize_false() {
        let v: bool = super::from_str("!f").unwrap();

        assert!(!v);
    }
    #[test]
    fn fail_deserialize_bool_trailing() {
        let v: super::Result<bool> = super::from_str("!ff");

        assert!(matches!(v, Err(_)));
    }
    #[test]
    fn deserialize_quoted_empty_string() {
        let v: String = super::from_str("''").unwrap();

        assert_eq!(v, "");
    }
    #[test]
    fn deserialize_quoted_string() {
        let v: String = super::from_str("'hello, rison'").unwrap();

        assert_eq!(v, "hello, rison");
    }
    #[test]
    fn deserialize_quoted_string_with_escapes() {
        let v: String = super::from_str("'hello, !'rison!'!!'").unwrap();

        assert_eq!(v, "hello, 'rison'!");
    }
    #[test]
    fn deserialize_ident_string() {
        let v: String = super::from_str("hellorison").unwrap();

        assert_eq!(v, "hellorison");
    }
    #[test]
    fn deserialize_none() {
        let v: Option<String> = super::from_str("!n").unwrap();

        assert_eq!(v, None);
    }
    #[test]
    fn deserialize_some_ident_string() {
        let v: Option<String> = super::from_str("hellorison").unwrap();

        assert_eq!(v, Some("hellorison".into()));
    }
    #[test]
    fn deserialize_empty_struct() {
        #[derive(serde::Deserialize)]
        struct Empty {}
        let _v: Empty = super::from_str("()").unwrap();
    }
    #[test]
    fn deserialize_struct() {
        #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
        struct Full {
            a: String,
            b: String,
        }
        let v: Full = super::from_str("(a:hello,b:world)").unwrap();

        assert_eq!(
            v,
            Full {
                a: "hello".into(),
                b: "world".into()
            }
        );
    }
    #[test]
    fn deserialize_struct_with_optional_present() {
        #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
        struct Full {
            a: String,
            b: Option<String>,
        }
        let v: Full = super::from_str("(a:hello,b:world)").unwrap();

        assert_eq!(
            v,
            Full {
                a: "hello".into(),
                b: Some("world".into())
            }
        );
    }
    #[test]
    fn deserialize_struct_with_optional_missing() {
        #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
        struct Full {
            a: String,
            b: Option<String>,
        }
        let v: Full = super::from_str("(a:hello)").unwrap();

        assert_eq!(
            v,
            Full {
                a: "hello".into(),
                b: None
            }
        );
    }
    #[test]
    fn deserialize_map() {
        let v: std::collections::HashMap<String, String> =
            super::from_str("(a:hello,b:world)").unwrap();

        let expected = vec![("a".into(), "hello".into()), ("b".into(), "world".into())]
            .into_iter()
            .collect();
        assert_eq!(v, expected);
    }
    #[test]
    fn deserialize_tuple() {
        let v: (String, String) = super::from_str("!(hello,world)").unwrap();

        assert_eq!(v, ("hello".into(), "world".into()));
    }
    #[test]
    fn deserialize_value_string() {
        let v: serde_json::Value = super::from_str("helloworld").unwrap();

        assert_eq!(v, serde_json::Value::String("helloworld".into()));
    }
    #[test]
    fn deserialize_value_map() {
        let v: serde_json::Value = super::from_str("(hello:!(a,b,c),world:'it works')").unwrap();

        assert_eq!(
            v,
            serde_json::json!({"hello": ["a", "b", "c"], "world": "it works"})
        );
    }
    #[test]
    fn deserialize_value_map_from_io() {
        let v: serde_json::Value =
            super::from_reader(b"(hello:!(a,b,c),world:'it works')" as &[_]).unwrap();

        assert_eq!(
            v,
            serde_json::json!({"hello": ["a", "b", "c"], "world": "it works"})
        );
    }
}
