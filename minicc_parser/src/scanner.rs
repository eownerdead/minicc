use std::process::exit;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub loc: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Plus,      // `+`
    Minus,     //`-`
    Asterisk,  //`*`
    Slash,     //`/`
    Percent,   // `%`
    Exclaim,   // `!`
    Lt,        // `<`
    Gt,        // `>`
    LtEq,      // `<=`
    GtEq,      // `>=`
    EqEq,      // `==`
    ExclaimEq, // `!=`
    LParen,    // `(`
    RParen,    // `)`
    LBrace,    // `{`
    RBrace,    // `}`
    Semi,      // `;`
    Eq,        // `=`

    If,
    Else,
    Int,
    Return,

    IntLit(i64), // Integer literals e.g. `123`

    Ident(String),

    Eof, // End Of File
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenKind::*;
        match self {
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Asterisk => write!(f, "*"),
            Slash => write!(f, "/"),
            Percent => write!(f, "%"),
            Exclaim => write!(f, "!"),
            Lt => write!(f, "<"),
            Gt => write!(f, ">"),
            LtEq => write!(f, "<="),
            GtEq => write!(f, ">="),
            EqEq => write!(f, "=="),
            ExclaimEq => write!(f, "!="),
            LParen => write!(f, "("),
            RParen => write!(f, ")"),
            LBrace => write!(f, "{{"),
            RBrace => write!(f, "}}"),
            Semi => write!(f, ";"),
            Eq => write!(f, "="),

            If => write!(f, "if"),
            Else => write!(f, "else"),
            Int => write!(f, "int"),
            Return => write!(f, "return"),

            IntLit(x) => write!(f, "{}", x),

            Ident(x) => write!(f, "{}", x),
            Eof => write!(f, "EOF"),
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

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip white spaces.
        while matches!(self.peek_char(), Some(c) if c.is_whitespace()) {
            self.next_char();
        }

        match self.peek_char() {
            None => None,

            Some('+') => {
                self.next_char();
                Some(Token { kind: TokenKind::Plus, loc: self.loc })
            }
            Some('-') => {
                self.next_char();
                Some(Token { kind: TokenKind::Minus, loc: self.loc })
            }
            Some('*') => {
                self.next_char();
                Some(Token { kind: TokenKind::Asterisk, loc: self.loc })
            }
            Some('/') => {
                self.next_char();
                Some(Token { kind: TokenKind::Slash, loc: self.loc })
            }
            Some('%') => {
                self.next_char();
                Some(Token { kind: TokenKind::Percent, loc: self.loc })
            }
            Some('!') => {
                self.next_char();
                if let Some('=') = self.peek_char() {
                    self.next_char();
                    Some(Token { kind: TokenKind::ExclaimEq, loc: self.loc })
                } else {
                    Some(Token { kind: TokenKind::Exclaim, loc: self.loc })
                }
            }
            Some('<') => {
                self.next_char();
                if let Some('=') = self.peek_char() {
                    self.next_char();
                    Some(Token { kind: TokenKind::LtEq, loc: self.loc })
                } else {
                    Some(Token { kind: TokenKind::Lt, loc: self.loc })
                }
            }
            Some('>') => {
                self.next_char();
                if let Some('=') = self.peek_char() {
                    self.next_char();
                    Some(Token { kind: TokenKind::GtEq, loc: self.loc })
                } else {
                    Some(Token { kind: TokenKind::Gt, loc: self.loc })
                }
            }
            Some('=') => {
                self.next_char();
                if let Some('=') = self.peek_char() {
                    self.next_char();
                    Some(Token { kind: TokenKind::EqEq, loc: self.loc })
                } else {
                    Some(Token { kind: TokenKind::Eq, loc: self.loc })
                }
            }
            Some('(') => {
                self.next_char();
                Some(Token { kind: TokenKind::LParen, loc: self.loc })
            }
            Some(')') => {
                self.next_char();
                Some(Token { kind: TokenKind::RParen, loc: self.loc })
            }
            Some('{') => {
                self.next_char();
                Some(Token { kind: TokenKind::LBrace, loc: self.loc })
            }
            Some('}') => {
                self.next_char();
                Some(Token { kind: TokenKind::RBrace, loc: self.loc })
            }
            Some(';') => {
                self.next_char();
                Some(Token { kind: TokenKind::Semi, loc: self.loc })
            }
            Some(c) if c.is_ascii_digit() => {
                Some(Token {
                    kind: TokenKind::IntLit(self.read_int()),
                    loc: self.loc,
                })
            }
            Some(c) if c.is_ascii_alphabetic() => Some(self.ident()),

            Some(c) => self.err(&format!("unknown token `{}`", c)),
        }
    }
}

impl<'a> Scanner<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { s: src.chars(), loc: 0 }
    }

    fn ident(&mut self) -> Token {
        let s = self.read_ident();
        match s.as_str() {
            "if" => Token { kind: TokenKind::If, loc: self.loc },
            "else" => Token { kind: TokenKind::Else, loc: self.loc },
            "int" => Token { kind: TokenKind::Int, loc: self.loc },
            "return" => Token { kind: TokenKind::Return, loc: self.loc },
            _ => Token { kind: TokenKind::Ident(s), loc: self.loc },
        }
    }

    fn read_ident(&mut self) -> String {
        let mut s = self.peek_char().unwrap().to_string();
        self.next_char();
        loop {
            match self.peek_char() {
                Some(c) if c.is_ascii_alphanumeric() => {
                    s.push(c);
                    self.next_char();
                }
                _ => return s,
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
