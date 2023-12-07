use super::error::{Error, ErrorKind, Result};

pub trait Read<'de> {
    fn next(&mut self) -> Result<Option<u8>> {
        let next = self.peek()?;
        if next.is_some() {
            self.discard();
        }
        Ok(next)
    }
    fn peek(&mut self) -> Result<Option<u8>>;
    fn discard(&mut self);
    // TODO: scratch and zero-copy optimisations
    fn parse_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<&'s str>;
    // TODO: scratch and zero-copy optimisations
    fn parse_ident(&mut self) -> Result<String>;
}

pub struct SliceRead<'a> {
    slice: &'a [u8],
    /// Index of the *next* byte that will be returned by next() or peek().
    index: usize,
}

impl<'a> SliceRead<'a> {
    /// Create a JSON input source to read from a slice of bytes.
    pub fn new(slice: &'a [u8]) -> Self {
        SliceRead { slice, index: 0 }
    }

    /// Parse a string from the input until a close-string delimiter
    /// # Safety
    /// Although this method is safe, and thus has no safety preconditions,
    /// safety elsewhere relies on the guarantee provided by this method that
    /// it will not transform the input stream such that valid utf-8 in the
    /// input becomes invalid in the output.
    fn parse_str_bytes<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<&'s [u8]> {
        let mut start = self.index;
        loop {
            while self.index < self.slice.len() && !is_control(self.slice[self.index]) {
                self.index += 1;
            }
            if self.index == self.slice.len() {
                return Err(Error {
                    kind: ErrorKind::Eof,
                });
            }
            match self.slice[self.index] {
                b'\'' => {
                    scratch.extend_from_slice(&self.slice[start..self.index]);
                    self.index += 1;
                    return Ok(scratch);
                }
                b'!' => {
                    scratch.extend_from_slice(&self.slice[start..self.index]);
                    self.index += 1;
                    scratch.push(
                        match self.next()?.ok_or(Error {
                            kind: ErrorKind::Eof,
                        })? {
                            c @ (b'!' | b'\'') => c,
                            _ => {
                                return Err(Error {
                                    kind: ErrorKind::Syntax,
                                })
                            }
                        },
                    );
                    start = self.index;
                }
                _ => {
                    return Err(Error {
                        kind: ErrorKind::Syntax,
                    })
                }
            }
        }
    }

    /// Parse an unquoted string from the input until a close-string delimiter
    /// # Safety
    /// Although this method is safe, and thus has no safety preconditions,
    /// safety elsewhere relies on the guarantee provided by this method that
    /// it will not transform the input stream such that valid utf-8 in the
    /// input becomes invalid in the output.
    fn parse_ident_bytes(&mut self) -> Result<&[u8]> {
        const NOT_ID_CHARS: &[u8] = b" '!:(),*@$";
        let start = self.index;
        while self.index < self.slice.len() && !NOT_ID_CHARS.contains(&self.slice[self.index]) {
            self.index += 1;
        }

        Ok(&self.slice[start..self.index])
    }
}

impl<'a> Read<'a> for SliceRead<'a> {
    fn peek(&mut self) -> Result<Option<u8>> {
        Ok(if self.index < self.slice.len() {
            let b = self.slice[self.index];
            Some(b)
        } else {
            None
        })
    }

    fn discard(&mut self) {
        self.index += 1;
    }

    fn parse_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<&'s str> {
        let bytes = self.parse_str_bytes(scratch)?;
        std::str::from_utf8(bytes).map_err(|_| Error {
            kind: ErrorKind::Syntax,
        })
    }
    fn parse_ident(&mut self) -> Result<String> {
        let bytes = self.parse_ident_bytes()?;

        Ok(std::str::from_utf8(bytes)
            .map_err(|_| Error {
                kind: ErrorKind::Syntax,
            })?
            .into())
    }
}

pub struct StrRead<'a> {
    delegate: SliceRead<'a>,
}

impl<'a> StrRead<'a> {
    /// Create a JSON input source to read from a slice of bytes.
    pub fn new(s: &'a str) -> Self {
        StrRead {
            delegate: SliceRead::new(s.as_bytes()),
        }
    }
}

impl<'a> Read<'a> for StrRead<'a> {
    fn peek(&mut self) -> Result<Option<u8>> {
        self.delegate.peek()
    }

    fn discard(&mut self) {
        self.delegate.discard()
    }

    fn parse_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<&'s str> {
        let bytes = self.delegate.parse_str_bytes(scratch)?;

        // # Safety
        // `parse_str_bytes` guarantees it will not transform
        // input such that valid utf-8 becomes invalid. StrRead's buffer
        // is guaranteed to be valid utf-8 by construction. The resulting
        // buffer is therefore valid utf-8, satisfying the safety preconditions
        // of `String::from_utf8_unchecked`
        Ok(unsafe { std::str::from_utf8_unchecked(bytes) })
    }
    fn parse_ident(&mut self) -> Result<String> {
        let bytes = self.delegate.parse_ident_bytes()?;

        // # Safety
        // `parse_ident_bytes` guarantees it will not transform
        // input such that valid utf-8 becomes invalid. StrRead's buffer
        // is guaranteed to be valid utf-8 by construction. The resulting
        // buffer is therefore valid utf-8, satisfying the safety preconditions
        // of `String::from_utf8_unchecked`.
        Ok(unsafe { std::str::from_utf8_unchecked(bytes) }.into())
    }
}

pub struct IoRead<I> {
    io: std::io::Bytes<I>,
    peeked: Option<u8>,
}

impl<I: std::io::Read> IoRead<I> {
    pub fn new(reader: I) -> Self {
        IoRead {
            io: reader.bytes(),
            peeked: None,
        }
    }
}

impl<'de, I> Read<'de> for IoRead<I>
where
    I: std::io::Read,
{
    fn peek(&mut self) -> Result<Option<u8>> {
        if let Some(ch) = self.peeked {
            return Ok(Some(ch));
        }

        let ch = self.io.next().transpose().map_err(|e| Error {
            kind: ErrorKind::Io(e),
        })?;

        self.peeked = ch;

        Ok(ch)
    }

    fn discard(&mut self) {
        self.peeked = None;
    }

    fn parse_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<&'s str> {
        loop {
            while let Some(ch) = self.peek()? {
                if is_control(ch) {
                    break;
                }
                scratch.push(ch);
                self.discard();
            }

            let Some(ch) = self.peek()? else {
                return Err(Error {
                    kind: ErrorKind::Eof,
                });
            };

            match ch {
                b'\'' => {
                    self.discard();
                    return std::str::from_utf8(scratch).map_err(|_| Error {
                        kind: ErrorKind::Syntax,
                    });
                }
                b'!' => {
                    self.discard();
                    scratch.push(
                        match self.next()?.ok_or(Error {
                            kind: ErrorKind::Eof,
                        })? {
                            c @ (b'!' | b'\'') => c,
                            _ => {
                                return Err(Error {
                                    kind: ErrorKind::Syntax,
                                })
                            }
                        },
                    );
                }
                _ => {
                    return Err(Error {
                        kind: ErrorKind::Syntax,
                    })
                }
            }
        }
    }

    fn parse_ident(&mut self) -> Result<String> {
        let mut scratch = Vec::new();
        const NOT_ID_CHARS: &[u8] = b" '!:(),*@$";

        while let Some(ch) = self.peek()? {
            if NOT_ID_CHARS.contains(&ch) {
                break;
            }
            scratch.push(ch);
            self.discard();
        }

        String::from_utf8(scratch).map_err(|_| Error {
            kind: ErrorKind::Syntax,
        })
    }
}

pub(crate) fn is_control(b: u8) -> bool {
    b <= 0x1f || b == b'\'' || b == b'!'
}
