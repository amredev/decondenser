use crate::error::{Error, Result};
use std::str::Chars;

pub(super) struct Cursor<'a> {
    /// Total number of bytes in the input.
    bytes: usize,
    chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        Self {
            bytes: input.len(),
            chars: input.chars(),
        }
    }

    pub(super) fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub(super) fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    pub(super) fn byte_offset(&self) -> usize {
        self.bytes - self.chars.as_str().len()
    }

    pub(super) fn strip_prefix(&mut self, prefix: &str) -> Option<usize> {
        let stripped = self.chars.as_str().strip_prefix(prefix)?;

        let start = self.byte_offset();
        self.chars = stripped.chars();

        Some(start)
    }
}
