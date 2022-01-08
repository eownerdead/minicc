use ast::Ast;
use minicc_ast as ast;
use minicc_ast::AstKind;

use super::scanner::{Token, TokenKind};

pub(crate) fn parse(tok: &[Token]) -> Ast {
    let (node, rest) = add(tok);
    skip(rest, TokenKind::Eof);

    node
}

/// ```ebnf
/// primary ::= [0..9]+
///           | "(" add ")"
/// ```
fn primary(tok: &[Token]) -> (Ast, &[Token]) {
    let (t, rest) = next(tok);
    match t.kind {
        TokenKind::Int(x) => {
            (
                Ast {
                    kind: AstKind::IntLit(ast::IntLit { val: x }),
                    span: t.span,
                },
                rest,
            )
        }

        TokenKind::LParen => {
            let (node, rest) = add(rest);
            let rest = skip(rest, TokenKind::RParen);

            (node, rest)
        }

        _ => {
            panic!(
                "expected expression, found `{}` at {}",
                t.kind, t.span.start.0
            )
        }
    }
}

/// ```ebnf
/// unary ::= ("+" | "-") unary
///         | primary
/// ```
fn unary(tok: &[Token]) -> (Ast, &[Token]) {
    let (t, rest) = next(tok);
    match t.kind {
        TokenKind::Plus => unary(rest),

        TokenKind::Minus => {
            let (expr, rest) = unary(rest);

            (
                Ast {
                    kind: AstKind::UnOp(ast::UnOp {
                        op: ast::OpUn::Neg,
                        expr: Box::new(expr),
                    }),
                    span: t.span.to(rest[0].span),
                },
                rest,
            )
        }

        _ => primary(tok),
    }
}

/// ```ebnf
/// mul ::= unary ("*" unary | "/" unary | "%" unary)*
/// ```
fn mul(tok: &[Token]) -> (Ast, &[Token]) {
    let (lhs, rest) = unary(tok);

    mul_rhs(rest, lhs)
}

fn mul_rhs(tok: &[Token], lhs: Ast) -> (Ast, &[Token]) {
    let (t, rest) = next(tok);
    let op = match t.kind {
        TokenKind::Asterisk => ast::OpBin::Mul,
        TokenKind::Slash => ast::OpBin::Div,
        TokenKind::Percent => ast::OpBin::Mod,
        _ => return (lhs, tok),
    };

    let (rhs, rest) = unary(rest);

    let lhs = Ast {
        kind: AstKind::BinOp(ast::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }),
        span: tok[0].span.to(rest[0].span),
    };

    mul_rhs(rest, lhs)
}

/// ```ebnf
/// add ::= mul ("+" mul | "-" mul)*
/// ```
fn add(tok: &[Token]) -> (Ast, &[Token]) {
    let (lhs, rest) = mul(tok);

    add_rhs(rest, lhs)
}

fn add_rhs(tok: &[Token], lhs: Ast) -> (Ast, &[Token]) {
    let (t, rest) = next(tok);
    let op = match t.kind {
        TokenKind::Plus => ast::OpBin::Add,
        TokenKind::Minus => ast::OpBin::Sub,
        _ => return (lhs, tok),
    };

    let (rhs, rest) = mul(rest);

    let lhs = Ast {
        kind: AstKind::BinOp(ast::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }),
        span: tok[0].span.to(rest[0].span),
    };

    add_rhs(rest, lhs)
}

fn next(tok: &[Token]) -> (&Token, &[Token]) {
    tok.split_first().unwrap()
}

fn skip(tok: &[Token], kind: TokenKind) -> &[Token] {
    let (t, rest) = next(tok);
    if t.kind != kind {
        panic!("expected `{}`, found `{}` at {}", kind, t.kind, t.span.start.0);
    } else {
        rest
    }
}
