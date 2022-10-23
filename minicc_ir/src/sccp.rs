use std::collections::BTreeMap;

use match_opt::match_opt;

use crate::{Func, Inst, OpBin, Operand};

pub fn sccp(func: &mut Func) {
    let mut reps = BTreeMap::new();

    for block in func.blocks.values() {
        for inst in &block.insts {
            match inst {
                Inst::Bin { op, dist, op1, op2 } => {
                    if let Some(c) = eval_bin(*op, *op1, *op2) {
                        reps.insert(*dist, Operand::Const(c));
                    }
                }
                _ => {}
            }
        }
    }

    for (from, to) in reps {
        for block in func.blocks.values_mut() {
            block.insts = block
                .insts
                .iter_mut()
                .filter(|inst| !matches!(inst.dist(), Some(d) if d == from))
                .map(|inst| {
                    inst.rename_var(from, to);
                    inst.clone()
                })
                .collect();
        }
    }
}

fn eval_bin(op: OpBin, op1: Operand, op2: Operand) -> Option<i64> {
    let op1 = match_opt!(op1, Operand::Const(c) => c)?;
    let op2 = match_opt!(op2, Operand::Const(c) => c)?;

    match op {
        OpBin::Add => Some(op1 + op2),
        OpBin::Sub => Some(op1 - op2),
        OpBin::Mul => Some(op1 * op2),
        OpBin::Div => Some(op1 / op2),
        OpBin::Mod => Some(op1 % op2),
        OpBin::Eq => todo!(),
        OpBin::Ne => todo!(),
        OpBin::Gt => todo!(),
        OpBin::Ge => todo!(),
        OpBin::Lt => todo!(),
        OpBin::Le => todo!(),
    }
}
