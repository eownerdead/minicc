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
    Comma,     // `,`

    If,
    Else,
    For,
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
            Comma => write!(f, ","),

            If => write!(f, "if"),
            Else => write!(f, "else"),
            For => write!(f, "for"),
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

        let loc = self.loc;
        let kind = match self.peek_char()? {
            '+' => {
                self.next_char();
                TokenKind::Plus
            }
            '-' => {
                self.next_char();
                TokenKind::Minus
            }
            '*' => {
                self.next_char();
                TokenKind::Asterisk
            }
            '/' => {
                self.next_char();
                TokenKind::Slash
            }
            '%' => {
                self.next_char();
                TokenKind::Percent
            }
            '!' => {
                self.next_char();
                if let Some('=') = self.peek_char() {
                    self.next_char();
                    TokenKind::ExclaimEq
                } else {
                    TokenKind::Exclaim
                }
            }
            '<' => {
                self.next_char();
                if let Some('=') = self.peek_char() {
                    self.next_char();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                self.next_char();
                if let Some('=') = self.peek_char() {
                    self.next_char();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            '=' => {
                self.next_char();
                if let Some('=') = self.peek_char() {
                    self.next_char();
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            '(' => {
                self.next_char();
                TokenKind::LParen
            }
            ')' => {
                self.next_char();
                TokenKind::RParen
            }
            '{' => {
                self.next_char();
                TokenKind::LBrace
            }
            '}' => {
                self.next_char();
                TokenKind::RBrace
            }
            ';' => {
                self.next_char();
                TokenKind::Semi
            }
            ',' => {
                self.next_char();
                TokenKind::Comma
            }
            c if c.is_ascii_digit() => TokenKind::IntLit(self.read_int()),
            c if c.is_ascii_alphabetic() => self.ident(),

            c => self.err(&format!("unknown token `{}`", c)),
        };

        Some(Token { kind, loc })
    }
}

impl<'a> Scanner<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { s: src.chars(), loc: 0 }
    }

    fn ident(&mut self) -> TokenKind {
        let s = self.read_ident();
        match s.as_str() {
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "int" => TokenKind::Int,
            "return" => TokenKind::Return,
            _ => TokenKind::Ident(s),
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
        eprintln!("{pos}: {msg}", pos = self.loc);
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
