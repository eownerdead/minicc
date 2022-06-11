use std::process::exit;

use ast::Ast;
use minicc_ast as ast;
use minicc_ast::AstKind;

use super::scanner::{Scanner, Token, TokenKind};

pub(crate) struct Parser<'a> {
    scanner: Scanner<'a>,
    tok: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut scanner: Scanner<'a>) -> Self {
        let tok = scanner.next();

        Self { scanner, tok }
    }

    pub fn parse(&mut self) -> Ast {
        self.skip(&TokenKind::LBrace);
        let node = self.compound_stmt();
        self.skip(&TokenKind::Eof);

        node
    }

    /// ```ebnf
    /// primary ::= [0-9]+
    ///           | [a-zA-Z][a-zA-Z0-9]*
    ///           | "(" add ")"
    /// ```
    fn primary(&mut self) -> Ast {
        let loc = self.cur().loc;

        match self.cur().kind.clone() {
            TokenKind::IntLit(x) => {
                self.next();
                Ast { kind: AstKind::IntLit(ast::IntLit { val: x }), loc }
            }
            TokenKind::Ident(i) => {
                self.next();
                Ast { kind: AstKind::Ref(ast::Ref { ident: i }), loc }
            }
            TokenKind::LParen => {
                self.next();
                let node = self.add();
                self.skip(&TokenKind::RParen);
                node
            }
            ref kind => {
                self.err(&format!("expected expression, found `{kind}`"));
            }
        }
    }

    /// ```ebnf
    /// unary ::= ("+" | "-") unary
    ///         | primary
    /// ```
    fn unary(&mut self) -> Ast {
        let loc = self.cur().loc;

        match self.cur().kind {
            TokenKind::Plus => {
                self.next();
                self.unary()
            }
            TokenKind::Minus => {
                self.next();
                let expr = self.unary();

                Ast {
                    kind: AstKind::UnOp(ast::UnOp {
                        op: ast::OpUn::Neg,
                        expr: Box::new(expr),
                    }),
                    loc,
                }
            }

            _ => self.primary(),
        }
    }

    /// ```ebnf
    /// mul ::= unary ("*" unary | "/" unary | "%" unary)*
    /// ```
    fn mul(&mut self) -> Ast {
        let lhs = self.unary();
        self.mul_rhs(lhs)
    }

    fn mul_rhs(&mut self, lhs: Ast) -> Ast {
        let loc = self.cur().loc;

        let op = match self.cur().kind {
            TokenKind::Asterisk => ast::OpBin::Mul,
            TokenKind::Slash => ast::OpBin::Div,
            TokenKind::Percent => ast::OpBin::Mod,
            _ => return lhs,
        };
        self.next();

        let rhs = self.unary();

        let lhs = Ast {
            kind: AstKind::BinOp(ast::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            loc,
        };

        self.mul_rhs(lhs)
    }

    /// ```ebnf
    /// add ::= mul ("+" mul | "-" mul)*
    /// ```
    fn add(&mut self) -> Ast {
        let lhs = self.mul();
        self.add_rhs(lhs)
    }

    fn add_rhs(&mut self, lhs: Ast) -> Ast {
        let loc = self.cur().loc;

        let op = match self.cur().kind {
            TokenKind::Plus => ast::OpBin::Add,
            TokenKind::Minus => ast::OpBin::Sub,
            _ => return lhs,
        };
        self.next();

        let rhs = self.mul();

        let lhs = Ast {
            kind: AstKind::BinOp(ast::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            loc,
        };

        self.add_rhs(lhs)
    }

    /// ```ebnf
    /// assign ::= add "=" assign
    /// ```
    fn assign(&mut self) -> Ast {
        let loc = self.cur().loc;

        let lhs = self.add();
        let op = match self.cur().kind {
            TokenKind::Eq => ast::OpBin::Asign,
            _ => return lhs,
        };
        self.next();

        let rhs = self.assign();

        Ast {
            kind: AstKind::BinOp(ast::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            loc,
        }
    }

    /// ```ebnf
    /// decl ::= [a-zA-Z][a-zA-Z0-9]*
    /// ```
    fn decl(&mut self) -> Ast {
        let loc = self.cur().loc;
        if let TokenKind::Ident(i) = self.cur().kind.clone() {
            self.next();
            Ast { kind: AstKind::Decl(ast::Decl { ident: i }), loc }
        } else {
            self.err("expected identifier")
        }
    }

    /// ```ebnf
    /// stmt ::= "{" compound_stmt
    ///        | "int" decl ";"
    ///        | "return" assign ";"
    ///        | assign ";"
    /// ```
    fn stmt(&mut self) -> Ast {
        let loc = self.cur().loc;

        match self.cur().kind {
            TokenKind::LBrace => {
                self.next();
                self.compound_stmt()
            }
            TokenKind::Int => {
                self.next();
                let node = self.decl();
                self.skip(&TokenKind::Semi);
                node
            }
            TokenKind::Return => {
                self.next();
                let expr = self.assign();
                self.skip(&TokenKind::Semi);
                Ast {
                    kind: AstKind::Return(ast::Return { expr: Box::new(expr) }),
                    loc,
                }
            }
            _ => {
                let node = self.assign();
                self.skip(&TokenKind::Semi);
                node
            }
        }
    }

    /// ```ebnf
    /// compound_stmt ::= stmt* "}"
    /// ```
    fn compound_stmt(&mut self) -> Ast {
        let loc = self.cur().loc;

        let mut item = Vec::new();
        loop {
            if self.cur().kind == TokenKind::RBrace {
                self.next();
                break;
            }

            let n = self.stmt();
            item.push(n);
        }

        Ast {
            kind: AstKind::CompoundStmt(ast::CompoundStmt { items: item }),
            loc,
        }
    }

    fn err(&self, msg: &str) -> ! {
        println!("{pos}: {msg}", pos = self.cur().loc);
        exit(1);
    }

    fn cur(&self) -> &Token {
        &self.tok
    }

    fn next(&mut self) -> &Token {
        self.tok = self.scanner.next();
        self.cur()
    }

    fn skip(&mut self, kind: &TokenKind) {
        if self.cur().kind != *kind {
            self.err(&format!(
                "expected `{expected}`, found `{found}`",
                expected = kind,
                found = self.cur().kind,
            ));
        }
        self.next();
    }
}
