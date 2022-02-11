use std::str::Chars;

use minicc_ast::{Pos, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Plus,     // `+`
    Minus,    //`-`
    Asterisk, //`*`
    Slash,    //`/`
    Percent,  // `%`
    LParen,   // `(`
    RParen,   // `)`
    LBrace,   // `{`
    RBrace,   // `}`
    Semi,     // `;`

    Int(i64), // Integer literals e.g. `123`

    Eof, // End Of File
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Asterisk => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::Int(x) => write!(f, "{}", x),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Scanner<'a> {
    // Cannot use enumerate iterator, because cannot get count after iterator
    // finished.
    s: Chars<'a>,
    loc: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { s: src.chars(), loc: 0 }
    }

    pub fn next(&mut self) -> Token {
        // Skip white spaces.
        while matches!(self.peek_char(), Some(c) if c.is_whitespace()) {
            self.next_char();
        }

        let begin = self.loc;
        match self.peek_char() {
            None => {
                Token {
                    kind: TokenKind::Eof,
                    span: Span { start: Pos(self.loc), end: Pos(self.loc) },
                }
            }

            Some('+') => {
                self.next_char();
                Token {
                    kind: TokenKind::Plus,
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }
            Some('-') => {
                self.next_char();
                Token {
                    kind: TokenKind::Minus,
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }
            Some('*') => {
                self.next_char();
                Token {
                    kind: TokenKind::Asterisk,
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }
            Some('/') => {
                self.next_char();
                Token {
                    kind: TokenKind::Slash,
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }
            Some('%') => {
                self.next_char();
                Token {
                    kind: TokenKind::Percent,
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }
            Some('(') => {
                self.next_char();
                Token {
                    kind: TokenKind::LParen,
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }
            Some(')') => {
                self.next_char();
                Token {
                    kind: TokenKind::RParen,
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }
            Some('{') => {
                self.next_char();
                Token {
                    kind: TokenKind::LBrace,
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }
            Some('}') => {
                self.next_char();
                Token {
                    kind: TokenKind::RBrace,
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }
            Some(';') => {
                self.next_char();
                Token {
                    kind: TokenKind::Semi,
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }
            Some(c) if c.is_ascii_digit() => {
                Token {
                    kind: TokenKind::Int(self.read_int()),
                    span: Span { start: Pos(begin), end: Pos(self.loc) },
                }
            }

            Some(c) => panic!("unknown token `{}` at {}", c, self.loc),
        }
    }

    fn read_int(&mut self) -> i64 {
        let mut s = String::new();

        loop {
            match self.peek_char() {
                Some(c) if c.is_ascii_digit() => {
                    s.push(c);
                    self.next_char();
                }
                _ => break,
            }
        }

        s.parse().unwrap()
    }

    fn next_char(&mut self) -> Option<char> {
        if let Some(c) = self.s.next() {
            self.loc += 1;
            Some(c)
        } else {
            None
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.s.clone().next()
    }
}
