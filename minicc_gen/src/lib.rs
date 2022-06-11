use std::collections::HashMap;
use std::io::Write;
use std::process::exit;

use minicc_ast as ast;

macro_rules! o {
    ($dst:expr) => {
        o!($dst,)
    };
    ($dst:expr, $($arg:tt)*) => {
        writeln!($dst, $($arg)*).unwrap()
    };
}

pub fn gen(f: &mut dyn Write, node: &ast::Ast) {
    let mut g = Gen { f, offset: 0, vars: Default::default() };
    g.prologue();
    g.gen(node);
    g.epilogue();
}

struct Gen<'a> {
    pub f: &'a mut dyn Write,
    pub offset: usize,
    pub vars: HashMap<String, usize>,
}

impl<'a> Gen<'a> {
    fn prologue(&mut self) {
        o!(self.f, "	.text");
        o!(self.f, ".Lmain:");
    }

    fn epilogue(&mut self) {
        o!(self.f, "	mov	%ebp, %esp");
        o!(self.f, "	pop	%ebp");
        o!(self.f, "	ret");
        o!(self.f);
        o!(self.f, "	.globl	main");
        o!(self.f, "	.type	main,@function");
        o!(self.f, "main:");
        o!(self.f, "	push	%ebp");
        o!(self.f, "	mov	%esp, %ebp");
        o!(self.f, "	sub	${}, %esp", self.offset);
        o!(self.f, "	jmp	.Lmain");
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
                    o!(self.f, "	mov	-{}(%ebp), %eax", offset);
                } else {
                    self.err(
                        node.loc,
                        &format!("cannot find value `{}`", n.ident),
                    );
                }
            }
            ast::AstKind::IntLit(n) => {
                o!(self.f, "	mov	${}, %eax", n.val);
            }
            ast::AstKind::UnOp(n) => {
                self.gen(&n.expr);

                match n.op {
                    ast::OpUn::Neg => {
                        o!(self.f, "	neg	%eax");
                    }
                }
            }
            ast::AstKind::BinOp(n) => {
                if n.op == ast::OpBin::Asign {
                    self.gen(&n.rhs);

                    if let ast::AstKind::Ref(l) = &n.lhs.kind {
                        if let Some(offset) = self.vars.get(&l.ident) {
                            o!(self.f, "	mov	%eax, -{}(%ebp)", offset)
                        } else {
                            self.err(node.loc, "cannot find value `{}`")
                        }
                    } else {
                        self.err(node.loc, "expression is not assignable");
                    }
                    return;
                }

                self.gen(&n.rhs);
                o!(self.f, "	push	%eax");
                self.gen(&n.lhs);

                o!(self.f, "	pop	%ecx");
                match n.op {
                    ast::OpBin::Add => {
                        o!(self.f, "	add	%ecx, %eax");
                    }
                    ast::OpBin::Sub => {
                        o!(self.f, "	sub	%ecx, %eax");
                    }
                    ast::OpBin::Mul => {
                        o!(self.f, "	imul	%ecx, %eax");
                    }
                    ast::OpBin::Div => {
                        o!(self.f, "	cltd");
                        o!(self.f, "	idiv	%ecx");
                    }
                    ast::OpBin::Mod => {
                        o!(self.f, "	cltd");
                        o!(self.f, "	idiv	%ecx");
                        o!(self.f, "	mov	%edx, %eax");
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
