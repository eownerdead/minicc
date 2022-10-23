use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Add,
};

use crate::{Block, BlockData, Func, Inst, Operand, Var};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarLoc {
    Reg(usize),
    Spilled,
}

pub fn regalloc(func: &mut Func) {
    let live = liveness(func);
    let result = live.keys().map(|&k| (k, VarLoc::Spilled)).collect();
    let mut regalloc = LinearScan {
        reg_size: 7,
        interval: live,
        alloced: result,
        reg_cnt: 0,
        active: BTreeSet::new(),
        free_reg: BTreeSet::new(),
    };
    regalloc.linear_scan();
    insert_alloca(func, regalloc.alloced);
}

fn insert_alloca(func: &mut Func, alloced: BTreeMap<Var, VarLoc>) {
    let entry = func.blocks.get_mut(&Block(0)).unwrap();

    entry.insts = alloced
        .iter()
        .filter(|(_, v)| matches!(v, VarLoc::Spilled))
        .map(|(k, _)| Inst::Alloca { dist: *k })
        .chain(entry.insts.iter().cloned())
        .collect();

    for (label, block) in &mut func.blocks {
        block.insts = block
            .insts
            .iter()
            .flat_map(|inst| {
                if let Some(&dist) = inst
                    .ops()
                    .iter()
                    .find(|i| matches!(alloced[i], VarLoc::Spilled))
                {
                    vec![
                        Inst::Load { dist, op1: Operand::Var(Var(666)) },
                        inst.clone(),
                    ]
                } else {
                    vec![inst.clone()]
                }
            })
            .collect();
    }
}

#[derive(Debug)]
struct LinearScan {
    reg_size: usize,
    interval: BTreeMap<Var, Interval>,
    alloced: BTreeMap<Var, VarLoc>,
    reg_cnt: usize,
    active: BTreeSet<Var>,
    free_reg: BTreeSet<Var>,
}

impl LinearScan {
    pub fn linear_scan(&mut self) {
        for &i in self.interval.clone().keys() {
            self.expire_old_intervals(i);
            if self.active.len() == self.reg_size {
                self.spill_at_intervals(i);
            } else {
                *self.alloced.get_mut(&i).unwrap() = self.new_reg();
                self.active.insert(i);
            }
        }
    }

    fn expire_old_intervals(&mut self, i: Var) {
        let mut active_rms = BTreeSet::new();
        let mut free_reg_adds = BTreeSet::new();
        for &j in &self.active {
            if self.interval[&j].end >= self.interval[&i].start {
                return;
            }
            active_rms.insert(j);
            free_reg_adds.insert(j);
        }
        self.active = self.active.intersection(&active_rms).cloned().collect();
        self.free_reg = self.free_reg.union(&free_reg_adds).cloned().collect();
    }

    fn spill_at_intervals(&mut self, i: Var) {
        let spill =
            *self.active.iter().max_by_key(|f| self.interval[f].start).unwrap();
        if self.interval[&spill].end >= self.interval[&i].end {
            *self.alloced.get_mut(&i).unwrap() = self.new_spill();
            self.active.remove(&spill);
            self.active.insert(i);
        } else {
            *self.alloced.get_mut(&i).unwrap() = self.new_spill()
        }
    }

    fn new_reg(&mut self) -> VarLoc {
        self.reg_cnt += 1;
        VarLoc::Reg(self.reg_cnt)
    }

    fn new_spill(&mut self) -> VarLoc {
        VarLoc::Spilled
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interval {
    pub start: usize,
    pub end: usize,
}

pub fn liveness(func: &Func) -> BTreeMap<Var, Interval> {
    let mut live_ranges = BTreeMap::new();

    let mut i = 0;
    for block in func.blocks.values() {
        for inst in &block.insts {
            if let Some(r) = inst.dist() {
                if live_ranges
                    .insert(
                        r,
                        Interval {
                            start: i,
                            end: 0, // dummy
                        },
                    )
                    .is_some()
                {
                    unreachable!()
                }
            }
            i += 1;
        }
    }

    for block in func.blocks.values().rev() {
        for inst in block.insts.iter().rev() {
            if !inst.ops().is_empty() {
                i -= 1;
            }
            for r in inst.ops() {
                live_ranges.entry(r).and_modify(|e| {
                    if e.end == 0 {
                        e.end = i;
                    }
                });
            }
        }
    }

    live_ranges
}
