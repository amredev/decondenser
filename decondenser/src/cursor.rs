use std::str::Chars;

pub(crate) struct Cursor<'a> {
    /// Total number of bytes in the input.
    bytes: usize,
    chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            bytes: input.len(),
            chars: input.chars(),
        }
    }

    pub(crate) fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub(crate) fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    pub(crate) fn byte_offset(&self) -> usize {
        self.bytes - self.chars.as_str().len()
    }

    pub(crate) fn strip_prefix(&mut self, prefix: &str) -> Option<usize> {
        let stripped = self.chars.as_str().strip_prefix(prefix)?;

        let start = self.byte_offset();
        self.chars = stripped.chars();

        Some(start)
    }

    pub(crate) fn find(&mut self, needle: char) -> Option<usize> {
        while let (byte_offset, Some(char)) = (self.byte_offset(), self.next()) {
            if char == needle {
                return Some(byte_offset);
            }
        }
        None
    }
}
