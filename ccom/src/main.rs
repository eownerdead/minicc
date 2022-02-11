use std::io::Read;

use minicc_ast as ast;

fn main() {
    let mut src = String::new();
    std::io::stdin().read_to_string(&mut src).unwrap();

    let node = minicc_parser::parse(&src);

    println!("	.text");
    println!("	.globl	main");
    println!("	.type	main,@function");
    println!("main:");
    gen(&node);
    println!("	ret");
}

fn gen(node: &ast::Ast) {
    match &node.kind {
        ast::AstKind::CompoundStmt(n) => {
            for i in &n.items {
                gen(i);
            }
        }
        ast::AstKind::IntLit(n) => {
            println!("	mov	${}, %eax", n.val);
        }
        ast::AstKind::UnOp(n) => {
            gen(&n.expr);

            match n.op {
                ast::OpUn::Neg => {
                    println!("	neg	%eax");
                }
            }
        }
        ast::AstKind::BinOp(n) => {
            gen(&n.rhs);
            println!("	push	%eax");
            gen(&n.lhs);

            println!("	pop	%ecx");
            match n.op {
                ast::OpBin::Add => {
                    println!("	add	%ecx, %eax");
                }
                ast::OpBin::Sub => {
                    println!("	sub	%ecx, %eax");
                }
                ast::OpBin::Mul => {
                    println!("	imul	%ecx, %eax");
                }
                ast::OpBin::Div => {
                    println!("	cltd");
                    println!("	idiv	%ecx");
                }
                ast::OpBin::Mod => {
                    println!("	cltd");
                    println!("	idiv	%ecx");
                    println!("	mov	%edx, %eax");
                }
            }
        }
    }
}
