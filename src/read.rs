use super::Error;

use super::Result;

pub trait Read<'de> {
    fn next(&mut self) -> Result<Option<u8>>;
    fn peek(&mut self) -> Result<Option<u8>>;
    fn discard(&mut self);
    // TODO: scratch and zero-copy optimisations
    fn parse_str(&mut self) -> Result<String>;
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
}

impl<'a> Read<'a> for SliceRead<'a> {
    fn next(&mut self) -> Result<Option<u8>> {
        Ok(if self.index < self.slice.len() {
            let b = self.slice[self.index];
            self.index += 1;
            Some(b)
        } else {
            None
        })
    }

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

    fn parse_str(&mut self) -> Result<String> {
        let mut scratch = Vec::new();
        let mut start = self.index;
        loop {
            while self.index < self.slice.len() && !is_control(self.slice[self.index]) {
                self.index += 1;
            }
            if self.index == self.slice.len() {
                return Err(Error {});
            }
            match self.slice[self.index] {
                b'\'' => {
                    scratch.extend_from_slice(&self.slice[start..self.index]);
                    self.index += 1;
                    return String::from_utf8(scratch).map_err(|_| Error {});
                }
                b'!' => {
                    scratch.extend_from_slice(&self.slice[start..self.index]);
                    self.index += 1;
                    scratch.push(match self.next()?.ok_or(Error {})? {
                        c @ (b'!' | b'\'') => c,
                        _ => return Err(Error {}),
                    });
                    start = self.index;
                }
                _ => return Err(Error {}),
            }
        }
    }
    fn parse_ident(&mut self) -> Result<String> {
        const NOT_ID_CHARS: &[u8] = b" '!:(),*@$";
        let start = self.index;
        while self.index < self.slice.len() && !NOT_ID_CHARS.contains(&self.slice[self.index]) {
            self.index += 1;
        }

        Ok(std::str::from_utf8(&self.slice[start..self.index])
            .map_err(|_| Error {})?
            .into())
    }
}

pub(crate) fn is_control(b: u8) -> bool {
    b <= 0x1f || b == b'\'' || b == b'!'
}
