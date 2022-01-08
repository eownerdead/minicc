pub mod parser;
pub mod scanner;

use minicc_ast::Ast;
use scanner::TokenKind;

pub fn parse(src: &str) -> Ast {
    let mut s = scanner::Scanner::new(src);

    let mut tok = Vec::new();
    loop {
        let t = s.next();
        tok.push(t.clone());
        if t.kind == TokenKind::Eof {
            break;
        }
    }

    debug_assert!(
        matches!(
            tok.last(),
            Some(scanner::Token { kind: TokenKind::Eof, span: _ })
        ),
        "'tok' must end with TokenKind::Eof"
    );

    parser::parse(&tok)
}
