use std::collections::HashMap;
use std::io::Write;
use std::ops::RangeFrom;
use std::process::exit;

use ir::{dom, regalloc};

use {minicc_ast as ast, minicc_ir as ir};

macro_rules! o {
    ($dst:expr) => {
        o!($dst,)
    };
    ($dst:expr, $($arg:tt)*) => {
        writeln!($dst, $($arg)*).unwrap()
    };
}

pub fn gen(f: &mut dyn Write, nodes: &[ast::Ast]) {
    let builder = ir::Builder::new();
    let mut g =
        Gen { f, label_cnt: 0.., curr_fn: Fn::new("".to_string()), builder };

    for i in nodes {
        g.gen(i);
    }

    let mut f = g.builder.mod_.funcs["main"].clone();

    //println!("original");
    println!("{f}");
    let preds = ir::pred::pred_blocks(&f);
    let doms = ir::dom::dom(&f, &preds);
    let dom_fr = ir::dom::dom_frontier(&f, &preds, &doms);
    //ir::to2addr(&mut f);
    ir::mem2reg::mem2reg(&mut f, &dom_fr);
    println!("mem2reg");
    println!("{f}");
    ir::sccp::sccp(&mut f);
    println!("sccp");
    println!("{f}");
    //regalloc(&mut f);
    //println!("{f}");
    ir::gen::GenAmd64.emit(f);
}

struct Gen<'a> {
    pub f: &'a mut dyn Write,
    pub label_cnt: RangeFrom<usize>,
    pub curr_fn: Fn,
    pub builder: ir::Builder,
}

struct Fn {
    pub ident: String,
    pub offset: isize,
    pub vars: HashMap<String, ir::Var>,
}

impl Fn {
    fn new(ident: String) -> Self {
        Self { ident, offset: 0, vars: HashMap::new() }
    }
}

impl<'a> Gen<'a> {
    fn gen(&mut self, node: &ast::Ast) -> Option<ir::Operand> {
        use ast::AstKind::*;
        match &node.kind {
            FnDecl(n) => self.fn_decl(n, node.loc),
            CompoundStmt(n) => self.compound_stmt(n, node.loc),
            If(n) => self.if_(n, node.loc),
            For(n) => self.for_(n, node.loc),
            Call(n) => self.call(n, node.loc),
            VarDecl(n) => self.var_decl(n, node.loc),
            Return(n) => self.return_(n, node.loc),
            Ref(n) => self.ref_(n, node.loc),
            IntLit(n) => self.int_lit(n, node.loc),
            UnOp(n) => self.un_op(n, node.loc),
            BinOp(n) => self.bin_op(n, node.loc),
        }
    }

    fn fn_decl(
        &mut self,
        node: &ast::FnDecl,
        _loc: usize,
    ) -> Option<ir::Operand> {
        self.curr_fn = Fn::new(node.ident.clone());
        self.builder.move_to_new_func("main".to_string());
        self.builder.move_to_new_block();

        // for (i, ident) in node.params.iter().enumerate() {
        // self.curr_fn.vars.insert(ident.clone(), 4 * (i as isize + 2));
        // }

        self.gen(&node.body);
        self.builder.push_inst(ir::Inst::Ret { op1: ir::Operand::Const(0) });
        None
    }

    fn compound_stmt(
        &mut self,
        node: &ast::CompoundStmt,
        _loc: usize,
    ) -> Option<ir::Operand> {
        for i in &node.items {
            self.gen(i);
        }
        None
    }

    fn if_(&mut self, node: &ast::If, _loc: usize) -> Option<ir::Operand> {
        let then = self.builder.new_block();
        let else_ = self.builder.new_block();
        let end = self.builder.new_block();

        let op1 = self.gen(&node.cond).unwrap();
        self.builder.push_inst(ir::Inst::Cond { op1, then, else_ });

        self.builder.move_to_block(then);
        self.gen(&node.then);
        self.builder.push_inst(ir::Inst::Jmp { label: end });

        self.builder.move_to_block(else_);
        if let Some(else_) = &node.else_ {
            self.gen(else_);
        }
        self.builder.push_inst(ir::Inst::Jmp { label: end });
        self.builder.move_to_block(end);

        None
    }

    fn for_(&mut self, node: &ast::For, _loc: usize) -> Option<ir::Operand> {
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
        None
    }

    fn call(&mut self, node: &ast::Call, _loc: usize) -> Option<ir::Operand> {
        for i in node.args.iter().rev() {
            self.gen(i);
            o!(self.f, "	push	%eax");
        }
        o!(self.f, "	call	{}", node.ident);
        o!(self.f, "	add	${}, %esp", node.args.len() * 4);
        todo!()
    }

    fn var_decl(
        &mut self,
        node: &ast::VarDecl,
        loc: usize,
    ) -> Option<ir::Operand> {
        let dist = self.builder.new_var();
        self.builder.push_inst(ir::Inst::Alloca { dist });
        if self.curr_fn.vars.insert(node.ident.clone(), dist).is_some() {
            self.err(
                loc,
                &format!(
                    "the variable `{}` is declared multiple times in the same \
                     function.",
                    node.ident
                ),
            )
        }
        None
    }

    fn return_(
        &mut self,
        node: &ast::Return,
        _loc: usize,
    ) -> Option<ir::Operand> {
        let op1 = self.gen(&node.expr).unwrap();
        self.builder.push_inst(ir::Inst::Ret { op1 });
        None
    }

    fn ref_(&mut self, node: &ast::Ref, loc: usize) -> Option<ir::Operand> {
        if let Some(&offset) = self.curr_fn.vars.get(&node.ident) {
            Some(offset.into())
        } else {
            self.err(loc, &format!("cannot find value `{}`", node.ident));
        }
    }

    fn int_lit(
        &mut self,
        node: &ast::IntLit,
        _loc: usize,
    ) -> Option<ir::Operand> {
        Some(ir::Operand::Const(node.val))
    }

    fn un_op(&mut self, node: &ast::UnOp, _loc: usize) -> Option<ir::Operand> {
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
        todo!()
    }

    fn bin_op(
        &mut self,
        node: &ast::BinOp,
        _loc: usize,
    ) -> Option<ir::Operand> {
        let op1 = self.gen(&node.lhs).unwrap();
        let op2 = self.gen(&node.rhs).unwrap();

        if node.op == ast::OpBin::Asign {
            self.builder.push_inst(ir::Inst::Store { op1, op2 });
            Some(op1)
        } else {
            let op = match node.op {
                ast::OpBin::Add => ir::OpBin::Add,
                ast::OpBin::Sub => ir::OpBin::Sub,
                ast::OpBin::Mul => ir::OpBin::Mul,
                ast::OpBin::Div => ir::OpBin::Div,
                ast::OpBin::Mod => ir::OpBin::Mod,
                ast::OpBin::Lt => ir::OpBin::Lt,
                ast::OpBin::Gt => ir::OpBin::Gt,
                ast::OpBin::Le => ir::OpBin::Le,
                ast::OpBin::Ge => ir::OpBin::Ge,
                ast::OpBin::Eq => ir::OpBin::Eq,
                ast::OpBin::Ne => ir::OpBin::Ne,
                ast::OpBin::Asign => unreachable!(),
            };

            let dist = self.builder.new_var();

            self.builder.push_inst(ir::Inst::Bin { op, dist, op1, op2 });

            Some(dist.into())
        }
    }

    fn next_label(&mut self) -> usize {
        self.label_cnt.next().unwrap()
    }

    fn err(&self, loc: usize, msg: &str) -> ! {
        eprintln!("{}: {}", loc, msg);
        exit(1);
    }
}
