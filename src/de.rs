//! Deserialize Rison data to Rust data structures

mod read;

use crate::error::{Code, Error, Result};
use read::Read;

/// A deserializer for Rison into Rust values
pub struct Deserializer<R> {
    read: R,
    scratch: Vec<u8>,
}

impl<R: std::io::Read> Deserializer<read::IoRead<R>> {
    /// Create a Rison deserializer from an `io::Read`
    pub fn from_reader(reader: R) -> Self {
        Self::new(read::IoRead::new(reader))
    }
}
impl<'a> Deserializer<read::SliceRead<'a>> {
    /// Create a Rison deserializer from a `&[u8]`
    pub fn from_slice(slice: &'a [u8]) -> Self {
        Self::new(read::SliceRead::new(slice))
    }
}
impl<'a> Deserializer<read::StrRead<'a>> {
    /// Create a Rison deserializer from a `&str`
    pub fn from_str(s: &'a str) -> Self {
        Self::new(read::StrRead::new(s))
    }
}

impl<'de, R: Read<'de>> Deserializer<R> {
    fn new(read: R) -> Self {
        Self {
            read,
            scratch: Vec::new(),
        }
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
                code: Code::TrailingChars,
                position: self.read.position().into(),
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
                    code: Code::EofMarker,
                    position: self.read.position().into(),
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
                            code: Code::EofList,
                            position: self.read.position().into(),
                        })? {
                            self.eat_char();
                        } else {
                            // TODO: Unreachable?
                            return Err(Error {
                                code: Code::TrailingChars,
                                position: self.read.position().into(),
                            });
                        };

                        ret
                    }
                    _ => Err(Error {
                        code: Code::InvalidMarker,
                        position: self.read.position().into(),
                    }),
                }
            }
            Some(b'-' | b'0'..=b'9') => {
                let mut f = String::new();
                while let Some(ch @ (b'-' | b'0'..=b'9' | b'.' | b'e')) = self.peek()? {
                    f.push(ch as char);
                    self.eat_char();
                }

                let v: f64 = f.parse().map_err(|_e| Error {
                    code: Code::InvalidNumber,
                    position: self.read.position().into(),
                })?;

                const MAX_INT: f64 = std::i32::MAX as _;
                const MIN_INT: f64 = std::i32::MIN as _;
                let truncated = v.trunc();
                if truncated == v && (MIN_INT..MAX_INT).contains(&truncated) {
                    visitor.visit_i32(truncated as i32)
                } else {
                    visitor.visit_f64(v)
                }
            }
            Some(b'\'') => {
                self.eat_char();

                self.scratch.clear();
                let s = self.read.parse_str(&mut self.scratch)?;

                match s {
                    read::Reference::Borrowed(borrowed) => visitor.visit_borrowed_str(borrowed),
                    read::Reference::Copied(copied) => visitor.visit_str(copied),
                }
            }
            Some(b'(') => {
                self.eat_char();

                let ret = visitor.visit_map(MapAccess::new(self));

                if let b')' = self.peek()?.ok_or(Error {
                    code: Code::EofObject,
                    position: self.read.position().into(),
                })? {
                    self.eat_char();
                } else {
                    // TODO: Unreachable?
                    return Err(Error {
                        code: Code::TrailingChars,
                        position: self.read.position().into(),
                    });
                };

                ret
            }
            Some(_) => {
                self.scratch.clear();
                let value = self.read.parse_ident(&mut self.scratch)?;
                match value {
                    read::Reference::Borrowed(borrowed) => visitor.visit_borrowed_str(borrowed),
                    read::Reference::Copied(copied) => visitor.visit_str(copied),
                }
            }
            None => Err(Error {
                code: Code::EofValue,
                position: self.read.position().into(),
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
                        code: Code::InvalidMarker,
                        position: self.read.position().into(),
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
                        code: Code::ExpectedObjectSepOrEnd,
                        position: self.de.read.position().into(),
                    });
                }
            }
            None => {
                return Err(Error {
                    code: Code::EofObject,
                    position: self.de.read.position().into(),
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
                    code: Code::ExpectedColon,
                    position: self.de.read.position().into(),
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
                        code: Code::ExpectedListSepOrEnd,
                        position: self.de.read.position().into(),
                    });
                }
            }
            None => {
                return Err(Error {
                    code: Code::EofList,
                    position: self.de.read.position().into(),
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

/// Deserialize an instance of `T` from a byte slice of Rison
pub fn from_slice<'a, T>(v: &'a [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    from_trait(read::SliceRead::new(v))
}

/// Deserialize an instance of `T` from a string of Rison
pub fn from_str<'a, T>(v: &'a str) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    from_trait(read::StrRead::new(v))
}

/// Deserialize an instance of `T` from an IO stream of Rison
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
    fn deserialize_integer() {
        let v: u32 = super::from_str("12").unwrap();

        assert_eq!(v, 12);
    }
    #[test]
    fn fail_deserialize_nonintegral_as_integer() {
        let v: super::Result<u32> = super::from_str("12.4");

        assert!(matches!(v, Err(_)));
    }
    #[test]
    fn deserialize_integral_float() {
        let v: f64 = super::from_str("12").unwrap();

        assert_eq!(v, 12.0);
    }
    #[test]
    fn deserialize_float() {
        let v: f64 = super::from_str("12.4").unwrap();

        assert_eq!(v, 12.4);
    }
    #[test]
    fn deserialize_float_exp() {
        let v: f64 = super::from_str("12.4e4").unwrap();

        assert_eq!(v, 12.4e4);
    }
    #[test]
    fn deserialize_float_neg_exp() {
        let v: f64 = super::from_str("12.4e-4").unwrap();

        assert_eq!(v, 12.4e-4);
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
