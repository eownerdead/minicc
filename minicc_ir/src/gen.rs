use std::collections::{BTreeMap, BTreeSet};

use crate::{Func, Inst, OpBin, Operand, Var, VarLoc};

const REGS: [&str; 7] =
    ["%rbx", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15"];

pub struct GenAmd64;

impl GenAmd64 {
    pub fn emit(&mut self, ir: Func) {
        println!("	.text");
        for (iblock, block) in ir.blocks {
            println!(".L{}:", iblock.0);
            for inst in &block.insts {
                match &inst {
                    Inst::Alloca { dist } => {}
                    Inst::Store { op1, op2 } => self.emit_mov(*op2, *op1),
                    Inst::Load { dist, op1 } => {
                        self.emit_mov(*op1, Operand::Var(*dist))
                    }
                    Inst::Bin { op, dist, op1, op2 } => {
                        let bin_op_gen = |op| {
                            self.emit_mov(*op2, Operand::Var(*dist));
                            println!(
                                "\t{}\t{}, {}",
                                op,
                                self.fmt_operand(*op1),
                                self.fmt_operand(Operand::Var(*dist))
                            );
                        };
                        match op {
                            OpBin::Add => bin_op_gen("add"),
                            OpBin::Sub => bin_op_gen("sub"),
                            OpBin::Mul => bin_op_gen("imul"),
                            OpBin::Div => {
                                self.emit_mov(*op2, Operand::Var(*dist));
                                println!(
                                    "\tmov\t{}, %rax",
                                    self.fmt_operand(*op1)
                                );
                                println!("\tcltd");
                                println!(
                                    "\tidiv\t{}",
                                    self.fmt_operand(Operand::Var(*dist))
                                );
                            }
                            OpBin::Mod => todo!(),
                            OpBin::Eq => todo!(),
                            OpBin::Ne => todo!(),
                            OpBin::Gt => todo!(),
                            OpBin::Ge => todo!(),
                            OpBin::Lt => todo!(),
                            OpBin::Le => todo!(),
                        }
                    }
                    Inst::Phi { dist: _, incomes: _ } => {},
                    Inst::Jmp { label } => println!("\tjmp\t.L{}", label.0),
                    Inst::Cond { op1, then, else_ } => {
                        if let Operand::Const(c) = op1 {
                            println!("\tmov\t${c}, %rax");
                            println!("\tcmp\t, %rax")
                        } else {
                            println!("\tcmp\t$0, {}", self.fmt_operand(*op1));
                        }
                        println!("\tjne\t.L{}", then.0);
                        println!("\tje\t.L{}", else_.0);
                    }
                    Inst::Ret { op1 } => {
                        println!("\tmov\t{}, %rax", self.fmt_operand(*op1));
                        println!("\tjmp\t.Lretmain");
                    }
                    Inst::Un { op, dist, op1 } => todo!(),
                }
            }
        }

        println!(".Lretmain:",);
        println!("	mov	%rbp, %rsp");
        println!("	pop	%rbp");
        println!("	ret");
        println!();
        println!("	.globl	main");
        println!("	.type	main, @function");
        println!("main:");
        println!("	push	%rbp");
        println!("	mov	%rsp, %rbp");
        println!("	sub	$8, %rsp");
        println!("	jmp	.L0");
    }

    fn emit_mov(&self, src: Operand, dist: Operand) {
        if let (Operand::Var(_), Operand::Var(_)) = (src, dist) {
            println!("\tmov\t{}, %rax", self.fmt_operand(src),);
            println!("\tmov\t%rax, {}", self.fmt_operand(dist));
        } else {
            println!(
                "\tmov\t{}, {}",
                self.fmt_operand(src),
                self.fmt_operand(dist)
            );
        }
    }

    fn fmt_operand(&self, operand: Operand) -> String {
        match operand {
            Operand::Var(v) => REGS[v.0].to_string(),
            Operand::Const(c) => format!("${c}"),
        }
    }
}
