use std::iter::Peekable;
use std::process::exit;

use ast::Ast;
use minicc_ast as ast;
use minicc_ast::AstKind;

use super::scanner::{Scanner, Token, TokenKind};

pub(crate) struct Parser<'a> {
    scanner: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        Self { scanner: scanner.peekable() }
    }

    pub fn parse(&mut self) -> Ast {
        self.skip(&TokenKind::LBrace);
        self.compound_stmt()
    }

    /// ```ebnf
    /// primary ::= [0-9]+
    ///           | [a-zA-Z][a-zA-Z0-9]*
    ///           | "(" eq ")"
    /// ```
    fn primary(&mut self) -> Ast {
        let loc = self.peek().loc;

        match self.peek().kind.clone() {
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
                let node = self.eq();
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
        let loc = self.peek().loc;

        let op = match self.peek().kind {
            TokenKind::Plus => {
                self.next();
                return self.unary();
            }
            TokenKind::Minus => {
                self.next();
                ast::OpUn::Neg
            }
            TokenKind::Exclaim => {
                self.next();
                ast::OpUn::LogNot
            }

            _ => return self.primary(),
        };

        Ast {
            kind: AstKind::UnOp(ast::UnOp { op, expr: Box::new(self.unary()) }),
            loc,
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
        let loc = self.peek().loc;

        let op = match self.peek().kind {
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
        let loc = self.peek().loc;

        let op = match self.peek().kind {
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
    /// rel := add ("<" add | ">" add | "<=" add | ">=" add)*
    /// ```
    fn rel(&mut self) -> Ast {
        let lhs = self.add();

        self.rel_rhs(lhs)
    }

    fn rel_rhs(&mut self, lhs: Ast) -> Ast {
        let loc = self.peek().loc;

        let op = match self.peek().kind {
            TokenKind::Lt => ast::OpBin::Lt,
            TokenKind::Gt => ast::OpBin::Gt,
            TokenKind::LtEq => ast::OpBin::Le,
            TokenKind::GtEq => ast::OpBin::Ge,
            _ => return lhs,
        };
        self.next();

        let rhs = self.add();

        let lhs = Ast {
            kind: AstKind::BinOp(ast::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            loc,
        };

        self.rel_rhs(lhs)
    }

    /// ```ebnf
    /// eq := rel ("==" rel | "!=" rel)*
    /// ```
    fn eq(&mut self) -> Ast {
        let lhs = self.rel();

        self.eq_rhs(lhs)
    }

    fn eq_rhs(&mut self, lhs: Ast) -> Ast {
        let loc = self.peek().loc;

        let op = match self.peek().kind {
            TokenKind::EqEq => ast::OpBin::Eq,
            TokenKind::ExclaimEq => ast::OpBin::Ne,
            _ => return lhs,
        };
        self.next();

        let rhs = self.rel();

        let lhs = Ast {
            kind: AstKind::BinOp(ast::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            loc,
        };

        self.eq_rhs(lhs)
    }

    /// ```ebnf
    /// assign ::= add "=" assign
    /// ```
    fn assign(&mut self) -> Ast {
        let loc = self.peek().loc;

        let lhs = self.eq();
        let op = match self.peek().kind {
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
        let loc = self.peek().loc;
        if let TokenKind::Ident(i) = self.peek().kind.clone() {
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
    ///        | "if" if_
    ///        | "dbg" "(" assign ")" ";"
    ///        | assign ";"
    /// ```
    fn stmt(&mut self) -> Ast {
        let loc = self.peek().loc;

        match self.peek().kind {
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
            TokenKind::If => {
                self.next();
                self.if_()
            }
            TokenKind::For => {
                self.next();
                self.for_()
            }
            TokenKind::Dbg => {
                self.next();
                self.skip(&TokenKind::LParen);
                let expr = self.assign();
                self.skip(&TokenKind::RParen);
                self.skip(&TokenKind::Semi);

                Ast {
                    kind: AstKind::Dbg(ast::Dbg { expr: Box::new(expr) }),
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
        let loc = self.peek().loc;

        let mut item = Vec::new();
        loop {
            if self.peek().kind == TokenKind::RBrace {
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

    /// ```ebnf
    /// if_ := "(" assign ")" stmt
    /// ```
    fn if_(&mut self) -> Ast {
        let loc = self.peek().loc;

        self.skip(&TokenKind::LParen);
        let cond = self.assign();
        self.skip(&TokenKind::RParen);
        let then = self.stmt();

        let else_ = if self.peek().kind == TokenKind::Else {
            self.next();
            Some(Box::new(self.stmt()))
        } else {
            None
        };

        Ast {
            kind: AstKind::If(ast::If {
                cond: Box::new(cond),
                then: Box::new(then),
                else_,
            }),
            loc,
        }
    }

    /// ```ebnf
    /// for_ := "(" assign? ";" assign? ";" assign? ")" stmt
    /// ```
    fn for_(&mut self) -> Ast {
        let loc = self.peek().loc;

        self.skip(&TokenKind::LParen);
        let init = if self.peek().kind == TokenKind::Semi {
            self.next();
            None
        } else {
            let init = Some(Box::new(self.assign()));
            self.skip(&TokenKind::Semi);
            init
        };

        let cond = if self.peek().kind == TokenKind::Semi {
            self.next();
            None
        } else {
            let cond = Some(Box::new(self.assign()));
            self.skip(&TokenKind::Semi);
            cond
        };

        let inc = if self.peek().kind == TokenKind::RParen {
            self.next();
            None
        } else {
            let inc = Some(Box::new(self.assign()));
            self.skip(&TokenKind::RParen);
            inc
        };

        let body = self.stmt();

        Ast {
            kind: AstKind::For(ast::For {
                init,
                cond,
                inc,
                body: Box::new(body),
            }),
            loc,
        }
    }

    fn err(&mut self, msg: &str) -> ! {
        println!("{pos}: {msg}", pos = self.peek().loc);
        exit(1);
    }

    fn peek(&mut self) -> &Token {
        self.scanner.peek().unwrap_or(&Token { kind: TokenKind::Eof, loc: 0 })
    }

    fn next(&mut self) -> Token {
        self.scanner.next().unwrap_or(Token { kind: TokenKind::Eof, loc: 0 })
    }

    fn skip(&mut self, kind: &TokenKind) {
        let k = self.peek().kind.clone();
        if k != *kind {
            self.err(&format!(
                "expected `{expected}`, found `{found}`",
                expected = kind,
                found = k,
            ));
        }
        self.next();
    }
}
