use std::process::exit;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub loc: usize,
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
    Eq,       // `=`

    Int,

    IntLit(i64), // Integer literals e.g. `123`

    Ident(String),

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
            TokenKind::Eq => write!(f, "="),

            TokenKind::Int => write!(f, "int"),

            TokenKind::IntLit(x) => write!(f, "{}", x),

            TokenKind::Ident(x) => write!(f, "{}", x),

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

        match self.peek_char() {
            None => Token { kind: TokenKind::Eof, loc: self.loc },

            Some('+') => {
                self.next_char();
                Token { kind: TokenKind::Plus, loc: self.loc }
            }
            Some('-') => {
                self.next_char();
                Token { kind: TokenKind::Minus, loc: self.loc }
            }
            Some('*') => {
                self.next_char();
                Token { kind: TokenKind::Asterisk, loc: self.loc }
            }
            Some('/') => {
                self.next_char();
                Token { kind: TokenKind::Slash, loc: self.loc }
            }
            Some('%') => {
                self.next_char();
                Token { kind: TokenKind::Percent, loc: self.loc }
            }
            Some('(') => {
                self.next_char();
                Token { kind: TokenKind::LParen, loc: self.loc }
            }
            Some(')') => {
                self.next_char();
                Token { kind: TokenKind::RParen, loc: self.loc }
            }
            Some('{') => {
                self.next_char();
                Token { kind: TokenKind::LBrace, loc: self.loc }
            }
            Some('}') => {
                self.next_char();
                Token { kind: TokenKind::RBrace, loc: self.loc }
            }
            Some(';') => {
                self.next_char();
                Token { kind: TokenKind::Semi, loc: self.loc }
            }
            Some('=') => {
                self.next_char();
                Token { kind: TokenKind::Eq, loc: self.loc }
            }
            Some(c) if c.is_ascii_digit() => {
                Token {
                    kind: TokenKind::IntLit(self.read_int()),
                    loc: self.loc,
                }
            }
            Some(c) if c.is_ascii_alphabetic() => self.ident(),

            Some(c) => self.err(&format!("unknown token `{}`", c)),
        }
    }

    fn ident(&mut self) -> Token {
        let mut s = self.peek_char().unwrap().to_string();
        self.next_char();
        loop {
            match self.peek_char() {
                Some(c) if c.is_ascii_alphanumeric() => {
                    s.push(c);
                    self.next_char();
                }
                _ => {
                    return match s.as_str() {
                        "int" => Token { kind: TokenKind::Int, loc: self.loc },
                        _ => Token { kind: TokenKind::Ident(s), loc: self.loc },
                    }
                }
            }
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

    fn err(&self, msg: &str) -> ! {
        println!("{pos}: {msg}", pos = self.loc);
        exit(1);
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
