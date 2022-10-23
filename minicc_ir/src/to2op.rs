use crate::{BlockData, Func, Inst, OpUn};

pub fn to2addr(func: &mut Func) {
    for block in func.blocks.values_mut() {
        let mut new_block = BlockData::new();
        for inst in &block.insts {
            if let &Inst::Bin { op, dist, op1, op2 } = inst {
                new_block.insts.append(&mut vec![
                    Inst::Un { op: OpUn::Copy, dist, op1 },
                    Inst::Bin { op, dist, op1: dist.into(), op2 },
                ])
            } else {
                new_block.insts.push(inst.clone());
            }
        }
        *block = new_block;
    }
}
