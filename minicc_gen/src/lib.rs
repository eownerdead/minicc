use std::collections::HashMap;
use std::io::Write;
use std::ops::RangeFrom;
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
    let mut g = Gen { f, label_cnt: 0.., offset: 0, vars: Default::default() };
    g.prologue();
    g.gen(node);
    g.epilogue();
}

struct Gen<'a> {
    pub f: &'a mut dyn Write,
    pub label_cnt: RangeFrom<usize>,
    pub offset: usize,
    pub vars: HashMap<String, usize>,
}

impl<'a> Gen<'a> {
    fn prologue(&mut self) {
        o!(self.f, "	.text");
        o!(self.f, ".Lmain:");
    }

    fn epilogue(&mut self) {
        // With out `return`, 0 should be returned.
        o!(self.f, "	mov	$0, %eax");
        o!(self.f, ".Lret:");
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
        use ast::AstKind::*;
        match &node.kind {
            CompoundStmt(n) => self.compound_stmt(n, node.loc),
            If(n) => self.if_(n, node.loc),
            For(n) => self.for_(n, node.loc),
            Decl(n) => self.decl(n, node.loc),
            Return(n) => self.return_(n, node.loc),
            Dbg(n) => self.dbg(n, node.loc),
            Ref(n) => self.ref_(n, node.loc),
            IntLit(n) => self.int_lit(n, node.loc),
            UnOp(n) => self.un_op(n, node.loc),
            BinOp(n) => self.bin_op(n, node.loc),
        }
    }

    fn compound_stmt(&mut self, node: &ast::CompoundStmt, _loc: usize) {
        for i in &node.items {
            self.gen(i);
        }
    }

    fn if_(&mut self, node: &ast::If, _loc: usize) {
        let elsel = self.next_label();
        let endl = self.next_label();

        self.gen(&node.cond);
        o!(self.f, "	cmp	$0, %eax");
        o!(self.f, "	je	.Lelse{elsel}");

        self.gen(&node.then);
        o!(self.f, "	jmp	.Lend{endl}");

        o!(self.f, ".Lelse{elsel}:");
        if let Some(else_) = &node.else_ {
            self.gen(else_);
        }
        o!(self.f, ".Lend{endl}:");
    }

    fn for_(&mut self, node: &ast::For, _loc: usize) {
        let beginl = self.next_label();
        let endl = self.next_label();

        if let Some(init) = &node.init {
            self.gen(init);
        }
        o!(self.f, ".Lbegin{beginl}:");
        if let Some(cond) = &node.cond {
            self.gen(cond);
            o!(self.f, "	cmp	$0, %eax");
            o!(self.f, "	je	.Lend{endl}");
        }
        self.gen(&node.body);
        if let Some(inc) = &node.inc {
            self.gen(inc);
        }
        o!(self.f, "	jmp	.Lbegin{beginl}");
        o!(self.f, ".Lend{endl}:");
    }

    fn decl(&mut self, node: &ast::Decl, _loc: usize) {
        self.offset += 4;
        self.vars.insert(node.ident.clone(), self.offset);
    }

    fn return_(&mut self, node: &ast::Return, _loc: usize) {
        self.gen(&node.expr);
        o!(self.f, "	jmp	.Lret");
    }

    fn dbg(&mut self, node: &ast::Dbg, _loc: usize) {
        self.gen(&node.expr);
        o!(self.f, "	push	%eax");
        o!(self.f, "	call	dbg");
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
            ast::OpUn::LogNot => {
                o!(self.f, "	cmp	$0, %eax");
                o!(self.f, "	sete	%al");
                o!(self.f, "	movzb	%al, %eax");
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
                return;
            }
            ast::OpBin::Sub => {
                o!(self.f, "	sub	%ecx, %eax");
                return;
            }
            ast::OpBin::Mul => {
                o!(self.f, "	imul	%ecx, %eax");
                return;
            }
            ast::OpBin::Div => {
                o!(self.f, "	cltd");
                o!(self.f, "	idiv	%ecx");
                return;
            }
            ast::OpBin::Mod => {
                o!(self.f, "	cltd");
                o!(self.f, "	idiv	%ecx");
                o!(self.f, "	mov	%edx, %eax");
                return;
            }
            _ => {}
        }

        o!(self.f, "	cmp	%ecx, %eax");
        match node.op {
            ast::OpBin::Lt => o!(self.f, "	setl	%al"),
            ast::OpBin::Gt => o!(self.f, "	setg	%al"),
            ast::OpBin::Le => o!(self.f, "	setle	%al"),
            ast::OpBin::Ge => o!(self.f, "	setge	%al"),
            ast::OpBin::Eq => o!(self.f, "	sete	%al"),
            ast::OpBin::Ne => o!(self.f, "	setne	%al"),
            _ => unreachable!("{:?}", node.op),
        }
        o!(self.f, "	movzb	%al, %eax");
    }

    fn next_label(&mut self) -> usize {
        self.label_cnt.next().unwrap()
    }

    fn err(&self, loc: usize, msg: &str) -> ! {
        eprintln!("{}: {}", loc, msg);
        exit(1);
    }
}
