use crate::cursor::Cursor;
use std::fmt;

#[derive(Clone, Copy)]
pub(crate) enum Token {
    Raw(usize),
    Escape(Escape),
}

impl Token {
    pub(crate) fn start(self) -> usize {
        match self {
            Self::Raw(start) => start,
            Self::Escape(escape) => escape.start,
        }
    }

    fn invalid_escape(start: usize) -> Self {
        Self::Escape(Escape {
            start,
            unescaped: Unescaped::Invalid,
        })
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(start) => write!(f, "qraw:{start}"),
            Self::Escape(escape) => write!(f, "qesc:{} {:?}", escape.start, escape.unescaped),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Escape {
    /// The input escape including the prefix character
    pub(crate) start: usize,

    /// `None` means the escape isn't valid
    pub(crate) unescaped: Unescaped,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Unescaped {
    Invalid,
    Char(char),

    /// Used to ignore a line break
    Ignore,
}

impl Unescaped {
    fn char_or_invalid(char: Option<char>) -> Self {
        match char {
            Some(c) => Self::Char(c),
            None => Self::Invalid,
        }
    }
}

pub(crate) struct Lexer<'i> {
    cursor: Cursor<'i>,
    escape_char: char,
    terminator: Option<&'i str>,
    state: State,
}

enum State {
    Normal,
    Escape(usize),
    End(usize),
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        match self.state {
            State::Normal => self.normal(),
            State::Escape(start) => Some(self.escape(start)),
            State::End(_) => None,
        }
    }
}

impl<'i> Lexer<'i> {
    pub(crate) fn new(cursor: Cursor<'i>) -> Self {
        Self {
            cursor,
            escape_char: '\\',
            terminator: None,
            state: State::Normal,
        }
    }

    pub(crate) fn with_escape_char(mut self, escape_char: char) -> Self {
        self.escape_char = escape_char;
        self
    }

    pub(crate) fn with_terminator(mut self, terminator: &'i str) -> Self {
        self.terminator = Some(terminator);
        self
    }

    fn escape(&mut self, start: usize) -> Token {
        self.state = State::Normal;

        let Some(char) = self.cursor.next() else {
            return Token::invalid_escape(start);
        };

        let unescaped = match char {
            'n' => Unescaped::Char('\n'),
            't' => Unescaped::Char('\t'),
            'r' => Unescaped::Char('\r'),
            '\\' => Unescaped::Char('\\'),
            '"' => Unescaped::Char('"'),
            '\'' => Unescaped::Char('\''),
            '#' => Unescaped::Char('#'),
            '$' => Unescaped::Char('$'),
            '0' => Unescaped::Char('\0'),
            'a' => Unescaped::Char('\x07'),
            'b' => Unescaped::Char('\x08'),
            'v' => Unescaped::Char('\x0B'),
            'f' => Unescaped::Char('\x0C'),
            'e' => Unescaped::Char('\x1B'),
            's' => Unescaped::Char(' '),
            '\n' => Unescaped::Ignore,
            'x' => Unescaped::char_or_invalid(self.hex()),
            'u' | 'U' => Unescaped::char_or_invalid(self.unicode_code_point()),
            _ => return Token::invalid_escape(start),
        };

        Token::Escape(Escape { start, unescaped })
    }

    fn hex(&mut self) -> Option<char> {
        let x1 = self.cursor.peek()?.to_digit(16)?;
        self.next();

        let Some(x2) = self.cursor.peek().and_then(|char| char.to_digit(16)) else {
            return char::from_u32(x1);
        };

        char::from_u32(x1 * 16 + x2)
    }

    fn unicode_code_point(&mut self) -> Option<char> {
        let braced = self.cursor.peek()? == '{';

        if braced {
            self.next();
        }

        let mut code_point = self.cursor.peek()?.to_digit(16)?;

        for _ in 0..8 {
            let Some(char) = self.cursor.peek() else {
                break;
            };

            let Some(digit) = char.to_digit(16) else {
                break;
            };

            self.next();

            code_point = code_point * 16 + digit;
        }

        if braced {
            if self.cursor.peek() != Some('}') {
                return None;
            }
            self.next();
        }

        char::from_u32(code_point)
    }

    fn normal(&mut self) -> Option<Token> {
        // Exit early if string is empty
        self.cursor.peek()?;

        let start = self.cursor.byte_offset();

        let escape_start = loop {
            if let Some(terminator) = self.terminator {
                if let Some(offset) = self.cursor.strip_prefix(terminator) {
                    self.state = State::End(offset);
                    return (start != offset).then_some(Token::Raw(start));
                }
            }

            let offset = self.cursor.byte_offset();

            let Some(char) = self.cursor.next() else {
                return (start != offset).then_some(Token::Raw(start));
            };

            if char == self.escape_char {
                break offset;
            }
        };

        if start == escape_start {
            return Some(self.escape(escape_start));
        }

        self.state = State::Escape(escape_start);

        Some(Token::Raw(start))
    }

    pub(crate) fn finish(self) -> LexingFinish<'i> {
        LexingFinish {
            terminator: match self.state {
                State::End(end) => Some(end),
                _ => None,
            },
            cursor: self.cursor,
        }
    }
}

pub(crate) struct LexingFinish<'i> {
    /// Offset of the terminating sequence if one was configured via
    /// [`Lexer::with_terminator()`].
    pub(crate) terminator: Option<usize>,

    /// The cursor in the state right after the last token was parsed.
    pub(crate) cursor: Cursor<'i>,
}
