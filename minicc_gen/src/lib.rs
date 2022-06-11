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
            ast::AstKind::CompoundStmt(n) => self.compound_stmt(n, node.loc),
            ast::AstKind::Decl(n) => self.decl(n, node.loc),
            ast::AstKind::Ref(n) => self.ref_(n, node.loc),
            ast::AstKind::IntLit(n) => self.int_lit(n, node.loc),
            ast::AstKind::UnOp(n) => self.un_op(n, node.loc),
            ast::AstKind::BinOp(n) => self.bin_op(n, node.loc),
        }
    }

    fn compound_stmt(&mut self, node: &ast::CompoundStmt, _loc: usize) {
        for i in &node.items {
            self.gen(i);
        }
    }

    fn decl(&mut self, node: &ast::Decl, _loc: usize) {
        self.offset += 4;
        self.vars.insert(node.ident.clone(), self.offset);
    }

    fn ref_(&mut self, node: &ast::Ref, loc: usize) {
        if let Some(offset) = self.vars.get(&node.ident) {
            o!(self.f, "	mov	-{}(%ebp), %eax", offset);
        } else {
            self.err(loc, &format!("cannot find value `{}`", node.ident));
        }
    }

    fn int_lit(&mut self, node: &ast::IntLit, _loc: usize) {
        o!(self.f, "	mov	${}, %eax", node.val);
    }

    fn un_op(&mut self, node: &ast::UnOp, _loc: usize) {
        self.gen(&node.expr);

        match node.op {
            ast::OpUn::Neg => {
                o!(self.f, "	neg	%eax");
            }
        }
    }

    fn bin_op(&mut self, node: &ast::BinOp, loc: usize) {
        if node.op == ast::OpBin::Asign {
            self.gen(&node.rhs);

            if let ast::AstKind::Ref(l) = &node.lhs.kind {
                if let Some(offset) = self.vars.get(&l.ident) {
                    o!(self.f, "	mov	%eax, -{}(%ebp)", offset)
                } else {
                    self.err(loc, &format!("cannot find value `{}`", l.ident))
                }
            } else {
                self.err(loc, "expression is not assignable");
            }
            return;
        }

        self.gen(&node.rhs);
        o!(self.f, "	push	%eax");
        self.gen(&node.lhs);

        o!(self.f, "	pop	%ecx");
        match node.op {
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

    fn err(&self, loc: usize, msg: &str) -> ! {
        println!("{}: {}", loc, msg);
        exit(1);
    }
}
