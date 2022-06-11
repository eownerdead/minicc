use std::collections::HashMap;
use std::io::Read;
use std::process::exit;

use minicc_ast as ast;

fn main() {
    let mut src = String::new();
    std::io::stdin().read_to_string(&mut src).unwrap();

    let node = minicc_parser::parse(&src);

    let mut gen = Gen::new();
    gen.prologue();
    gen.gen(&node);
    gen.epilogue();
}

pub struct Gen {
    pub offset: usize,
    pub vars: HashMap<String, usize>,
}

impl Gen {
    fn new() -> Self {
        Self { offset: 0, vars: Default::default() }
    }

    fn prologue(&mut self) {
        println!("	.text");
        println!(".Lmain:");
    }

    fn epilogue(&mut self) {
        println!("	mov	%ebp, %esp");
        println!("	pop	%ebp");
        println!("	ret");
        println!();
        println!("	.globl	main");
        println!("	.type	main,@function");
        println!("main:");
        println!("	push	%ebp");
        println!("	mov	%esp, %ebp");
        println!("	sub	${}, %esp", self.offset);
        println!("	jmp	.Lmain");
    }

    fn gen(&mut self, node: &ast::Ast) {
        match &node.kind {
            ast::AstKind::CompoundStmt(n) => {
                for i in &n.items {
                    self.gen(i);
                }
            }
            ast::AstKind::Decl(n) => {
                self.offset += 4;
                self.vars.insert(n.ident.clone(), self.offset);
            }
            ast::AstKind::Ref(n) => {
                if let Some(offset) = self.vars.get(&n.ident) {
                    println!("	mov	-{}(%ebp), %eax", offset);
                } else {
                    self.err(
                        node.loc,
                        &format!("cannot find value `{}`", n.ident),
                    );
                }
            }
            ast::AstKind::IntLit(n) => {
                println!("	mov	${}, %eax", n.val);
            }
            ast::AstKind::UnOp(n) => {
                self.gen(&n.expr);

                match n.op {
                    ast::OpUn::Neg => {
                        println!("	neg	%eax");
                    }
                }
            }
            ast::AstKind::BinOp(n) => {
                if n.op == ast::OpBin::Asign {
                    self.gen(&n.rhs);

                    if let ast::AstKind::Ref(l) = &n.lhs.kind {
                        if let Some(offset) = self.vars.get(&l.ident) {
                            println!("	mov	%eax, -{}(%ebp)", offset)
                        } else {
                            self.err(node.loc, "cannot find value `{}`")
                        }
                    } else {
                        self.err(node.loc, "expression is not assignable");
                    }
                    return;
                }

                self.gen(&n.rhs);
                println!("	push	%eax");
                self.gen(&n.lhs);

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
                    _ => unreachable!(),
                }
            }
        }
    }

    fn err(&self, loc: usize, msg: &str) -> ! {
        println!("{}: {}", loc, msg);
        exit(1);
    }
}
