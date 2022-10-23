use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

use match_opt::match_opt;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mod {
    pub funcs: HashMap<String, Func>,
}

impl fmt::Display for Mod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, func) in &self.funcs {
            writeln!(f, "func {i}() {{")?;
            writeln!(f, "{func}")?;
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

impl Mod {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Func {
    pub blocks: BTreeMap<Block, BlockData>,
}

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, block) in &self.blocks {
            writeln!(f, "{}:", i)?;
            writeln!(f, "{}", block)?;
        }
        Ok(())
    }
}

impl Func {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rename_var(&mut self, from: Var, to: Operand) {
        for block in self.blocks.values_mut() {
            block.rename_var(from, to);
        }
    }

    pub fn new_block(&mut self) -> Block {
        let block =
            self.blocks.keys().max().map_or(Block(0), |b| Block(b.0 + 1));
        self.blocks.insert(block, BlockData::new());
        block
    }

    pub fn all_vars(&self) -> BTreeSet<Var> {
        self.blocks
            .values()
            .flat_map(|i| &i.insts)
            .filter_map(|i| i.dist())
            .collect()
    }

    pub fn new_var(&self) -> Var {
        self.all_vars().iter().max().map_or(Var(0), |v| Var(v.0 + 1))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loc {
    pub block: Block,
    pub inst_idx: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockData {
    pub insts: Vec<Inst>,
}

impl fmt::Display for BlockData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in &self.insts {
            writeln!(f, "\t{}", i)?;
        }
        Ok(())
    }
}

impl BlockData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rename_var(&mut self, from: Var, to: Operand) {
        for inst in &mut self.insts {
            inst.rename_var(from, to)
        }
    }

    pub fn succ(&self) -> Vec<Block> {
        match self.insts.iter().find(|i| i.terminator()).unwrap() {
            Inst::Jmp { label } => vec![*label],
            Inst::Cond { then, else_, .. } => vec![*then, *else_],
            Inst::Ret { .. } => vec![],
            _ => panic!("does not have terminator instruction"),
        }
    }

    pub fn terminator(&self) -> &Inst {
        self.insts.iter().find(|i| i.terminator()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inst {
    Alloca { dist: Var },
    Store { op1: Operand, op2: Operand },
    Load { dist: Var, op1: Operand },
    Bin { op: OpBin, dist: Var, op1: Operand, op2: Operand },
    Un { op: OpUn, dist: Var, op1: Operand },
    Phi { dist: Var, incomes: BTreeMap<Block, Operand> },
    Jmp { label: Block },
    Cond { op1: Operand, then: Block, else_: Block },
    Ret { op1: Operand },
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alloca { dist } => write!(f, "{dist} = alloca"),
            Self::Store { op1, op2 } => write!(f, "store {op1}, {op2}"),
            Self::Load { dist, op1 } => write!(f, "{dist} = load {op1}"),
            Self::Bin { op, dist, op1, op2 } => {
                write!(f, "{dist} = {op} {op1}, {op2}")
            }
            Self::Un { op, dist, op1 } => {
                write!(f, "{dist} = {op} {op1}")
            }
            Self::Phi { dist, incomes } => {
                write!(f, "{dist} = phi {incomes:?}")
            }
            Self::Jmp { label } => write!(f, "jmp {label}"),
            Self::Cond { op1, then, else_ } => {
                write!(f, "cond {op1}, {then}, {else_}")
            }
            Self::Ret { op1 } => write!(f, "ret {op1}"),
        }
    }
}

impl Inst {
    pub fn rename_var(&mut self, from: Var, to: Operand) {
        let re_dist = |var| {
            if let Operand::Var(to) = to {
                if var == from { to } else { var }
            } else {
                var
            }
        };
        let re_op = |op| if op == from.into() { to.into() } else { op };

        *self = match &self {
            Inst::Alloca { dist } => Inst::Alloca { dist: re_dist(*dist) },
            Inst::Store { op1, op2 } => {
                Inst::Store { op1: re_op(*op1), op2: re_op(*op2) }
            }
            Inst::Load { dist, op1 } => {
                Inst::Load { dist: re_dist(*dist), op1: re_op(*op1) }
            }
            Inst::Bin { op, dist, op1, op2 } => Inst::Bin {
                op: *op,
                dist: re_dist(*dist),
                op1: re_op(*op1),
                op2: re_op(*op2),
            },
            Inst::Un { op, dist, op1 } => {
                Inst::Un { op: *op, dist: re_dist(*dist), op1: re_op(*op1) }
            }
            Inst::Phi { dist, incomes } => {
                Inst::Phi { dist: re_dist(*dist), incomes: incomes.clone() }
            }
            Inst::Jmp { .. } => self.clone(),
            Inst::Cond { op1, then, else_ } => {
                Inst::Cond { op1: re_op(*op1), then: *then, else_: *else_ }
            }
            Inst::Ret { op1 } => Inst::Ret { op1: re_op(*op1) },
        };
    }

    pub fn dist(&self) -> Option<Var> {
        match *self {
            Self::Alloca { dist } => Some(dist),
            Self::Load { dist, .. } => Some(dist),
            Self::Bin { dist, .. } => Some(dist),
            Self::Phi { dist, .. } => Some(dist),
            _ => None,
        }
    }

    pub fn ops(&self) -> Vec<Var> {
        match self {
            Self::Store { op1, op2 } => match (op1, op2) {
                (Operand::Var(r1), Operand::Var(r2)) => vec![*r1, *r2],
                (Operand::Var(r), _) | (_, Operand::Var(r)) => vec![*r],
                _ => vec![],
            },
            Self::Load { op1, .. } => {
                match_opt!(op1, &Operand::Var(r) => vec![r]).unwrap_or_default()
            }
            Self::Bin { op1, op2, .. } => match (op1, op2) {
                (&Operand::Var(r1), &Operand::Var(r2)) => vec![r1, r2],
                (&Operand::Var(r), _) | (_, &Operand::Var(r)) => vec![r],
                _ => vec![],
            },
            Self::Phi { incomes, .. } => incomes
                .values()
                .filter_map(|i| match_opt!(i, Operand::Var(r) => *r))
                .collect(),
            Self::Cond { op1, .. } => {
                match_opt!(op1, Operand::Var(r) => vec![*r]).unwrap_or_default()
            }
            Self::Ret { op1 } => {
                match_opt!(op1, Operand::Var(r) => vec![*r]).unwrap_or_default()
            }
            _ => Vec::new(),
        }
    }

    pub fn terminator(&self) -> bool {
        matches!(self, Inst::Jmp { .. } | Inst::Cond { .. } | Inst::Ret { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Var(Var),
    Const(i64),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Var(v) => write!(f, "{v}"),
            Operand::Const(c) => write!(f, "{c}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpBin {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

impl fmt::Display for OpBin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_ascii_lowercase())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpUn {
    Copy,
}

impl fmt::Display for OpUn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_ascii_lowercase())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Var(pub usize);

impl From<Var> for Operand {
    fn from(v: Var) -> Self {
        Operand::Var(v)
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Block(pub usize);

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}", self.0)
    }
}

impl From<usize> for Block {
    fn from(n: usize) -> Self {
        Self(n)
    }
}

impl From<Block> for usize {
    fn from(x: Block) -> Self {
        x.0
    }
}
