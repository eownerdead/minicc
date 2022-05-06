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
    /// primary ::= [0..9]+
    ///           | "(" add ")"
    /// ```
    fn primary(&mut self) -> Ast {
        let start = self.cur().span;

        match self.cur().kind {
            TokenKind::Int(x) => {
                self.next();
                Ast {
                    kind: AstKind::IntLit(ast::IntLit { val: x }),
                    span: start,
                }
            }
            TokenKind::LParen => {
                self.next();
                let node = self.add();
                self.skip(&TokenKind::RParen);
                Ast { span: start.to(self.cur().span), ..node }
            }
            ref kind => {
                panic!(
                    "expected expression, found `{kind}` at {pos}",
                    pos = start.start.0,
                )
            }
        }
    }

    /// ```ebnf
    /// unary ::= ("+" | "-") unary
    ///         | primary
    /// ```
    fn unary(&mut self) -> Ast {
        let start = self.cur().span;

        match self.cur().kind {
            TokenKind::Plus => {
                self.next();
                let node = self.unary();
                Ast { span: start.to(self.cur().span), ..node }
            }
            TokenKind::Minus => {
                self.next();
                let expr = self.unary();

                Ast {
                    kind: AstKind::UnOp(ast::UnOp {
                        op: ast::OpUn::Neg,
                        expr: Box::new(expr),
                    }),
                    span: start.to(self.cur().span),
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
        let start = self.cur().span;

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
            span: start.to(self.cur().span),
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
        let start = self.cur().span;

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
            span: start.to(self.cur().span),
        };

        self.add_rhs(lhs)
    }

    /// ```ebnf
    /// stmt ::= "{" compound_stmt
    ///        | add ";"
    /// ```
    fn stmt(&mut self) -> Ast {
        if self.cur().kind == TokenKind::LBrace {
            self.compound_stmt()
        } else {
            let node = self.add();
            self.skip(&TokenKind::Semi);

            node
        }
    }

    /// ```ebnf
    /// compound_stmt ::= stmt* "}"
    /// ```
    fn compound_stmt(&mut self) -> Ast {
        let start = self.cur().span;

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
            span: start.to(self.cur().span),
        }
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
            panic!(
                "expected `{expected}`, found `{found}` at {pos}",
                expected = kind,
                found = self.cur().kind,
                pos = self.cur().span.start.0
            );
        }
        self.next();
    }
}
