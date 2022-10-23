use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;
use typed_index_collections::TiVec;

use crate::inst::{Loc, Operand, Var};
use crate::{Block, BlockData, Func, Inst};

pub fn mem2reg(
    func: &mut Func,
    dom_frontier: &BTreeMap<Block, BTreeSet<Block>>,
) {
    for reg in collect_promotable(func) {
        if stored_only_once(func, reg) {
            single_stored(func, reg);
        } else {
            let mut r = insert_phi(func, dom_frontier, reg);
            rename(func, &mut r, reg);
        }
    }
}

fn rename(func: &mut Func, phis: &mut Phis, reg: Var) {
    let mut rn = Rename { visited: BTreeSet::new(), phis };
    rn.rename1(func, Block(0), reg);
    debug_assert!(rn.visited.iter().all(|i| func.blocks.get(i).is_some()));

    rewrite(func, rn.phis, reg);
}

type Phis = BTreeMap<Block, BTreeMap<Block, Operand>>;

struct Rename<'a> {
    phis: &'a mut Phis,
    visited: BTreeSet<Block>,
}

impl<'a> Rename<'a> {
    fn rename1(&mut self, func: &mut Func, block: Block, reg: Var) {
        self.visited.insert(block);

        for inst in &func.blocks[&block].insts {
            match &inst {
                Inst::Store { op1: Operand::Var(r), op2 } if *r == reg => {
                    for succ in func.blocks[&block].succ() {
                        if let Some(phi) = self.phis.get_mut(&succ) {
                            phi.insert(block, *op2);
                        }
                    }
                }
                _ => {}
            }
        }

        for succ in func.blocks[&block].succ() {
            if !self.visited.contains(&succ) {
                self.rename1(func, succ, reg);
            }
        }
    }
}

fn rewrite(func: &mut Func, phis: &Phis, reg: Var) {
    for (block, phi) in phis {
        func.blocks
            .get_mut(block)
            .unwrap()
            .insts
            .insert(0, Inst::Phi { dist: reg, incomes: phi.clone() });
    }

    for block in func.blocks.values_mut() {
        block.insts.retain(|inst|
            !matches!(inst, Inst::Alloca { dist } if *dist == reg)
            && !matches!(inst, Inst::Store { op1, .. } if *op1 == reg.into()));
    }
}

fn insert_phi(
    func: &mut Func,
    dom_frontier: &BTreeMap<Block, BTreeSet<Block>>,
    reg: Var,
) -> Phis {
    let mut phis = BTreeMap::new();

    for (label, block) in &func.blocks {
        for inst in &block.insts {
            if matches!(&inst, Inst::Store { op1: Operand::Var(r), .. } if *r == reg)
            {
                for df in &dom_frontier[label] {
                    phis.insert(*df, Default::default());
                }
            }
        }
    }

    phis
}

fn single_stored(func: &mut Func, reg: Var) {
    let mut src = None;

    for block in func.blocks.values_mut() {
        let mut new_block = BlockData::new();
        for inst in &mut block.insts {
            if let Inst::Store { op1, op2, .. } = inst {
                if *op1 == reg.into() {
                    src = Some(*op2);
                    continue;
                }
            }
            if matches!(inst, Inst::Alloca { dist } if *dist == reg) {
                continue;
            }
            if let Some(src) = src {
                inst.rename_var(reg, src);
            }
            new_block.insts.push(inst.clone());
        }
        *block = new_block;
    }
}

fn stored_only_once(func: &Func, reg: Var) -> bool {
    func.blocks
        .values()
        .flat_map(|i| &i.insts)
        .filter(|i| matches!(i, Inst::Store {op1, ..} if *op1 == reg.into()))
        .count()
        == 1
}

fn collect_promotable(func: &Func) -> BTreeSet<Var> {
    let mut canditates = BTreeSet::new();

    for block in func.blocks.values() {
        for i in &block.insts {
            if let Inst::Alloca { dist } = i {
                canditates.insert(*dist);
            }
        }
    }

    canditates
}
