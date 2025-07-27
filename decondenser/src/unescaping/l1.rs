use crate::cursor::Cursor;

#[derive(Clone, Copy)]
pub(super) enum Token {
    Raw(usize),
    Escape(Escape),
}

impl Token {
    pub(super) fn start(self) -> usize {
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

#[derive(Clone, Copy)]
pub(super) struct Escape {
    /// The input escape including the prefix character
    pub(super) start: usize,

    /// `None` means the escape isn't valid
    pub(super) unescaped: Unescaped,
}

#[derive(Clone, Copy)]
pub(super) enum Unescaped {
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

pub(super) struct Lexer<'i> {
    cursor: Cursor<'i>,
    state: State,
}

enum State {
    Normal,
    Escape(usize),
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        match self.state {
            State::Normal => self.normal(),
            State::Escape(start) => Some(self.escape(start)),
        }
    }
}

impl<'i> Lexer<'i> {
    pub(super) fn new(input: &'i str) -> Self {
        Self {
            cursor: Cursor::new(input),
            state: State::Normal,
        }
    }

    fn escape(&mut self, start: usize) -> Token {
        self.state = State::Normal;

        let Some(char) = self.cursor.next() else {
            return Token::invalid_escape(start);
        };

        let unescaped = match char {
            'n' => Unescaped::Char('\n'),
            'r' => Unescaped::Char('\r'),
            't' => Unescaped::Char('\t'),
            '\\' => Unescaped::Char('\\'),
            '\0' => Unescaped::Char('\0'),
            '\n' => Unescaped::Ignore,
            'x' => Unescaped::char_or_invalid(self.hex()),
            'u' | 'U' => Unescaped::char_or_invalid(self.unicode_code_point()),
            _ => return Token::invalid_escape(start),
        };

        let escape = Escape { start, unescaped };

        Token::Escape(escape)
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

        let escape_char = '\\';

        let Some(escape) = self.cursor.find(escape_char) else {
            return Some(Token::Raw(start));
        };

        if start == self.cursor.byte_offset() {
            return Some(self.escape(escape));
        }

        self.state = State::Escape(escape);

        Some(Token::Raw(start))
    }
}
