pub mod parser;
pub mod scanner;

use minicc_ast::Ast;

pub fn parse(src: &str) -> Vec<Ast> {
    let scanner = scanner::Scanner::new(src);

    let mut p = parser::Parser::new(scanner);
    p.parse()
}
