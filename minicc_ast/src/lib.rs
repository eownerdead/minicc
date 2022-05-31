#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ast {
    pub kind: AstKind,
    pub loc: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstKind {
    CompoundStmt(CompoundStmt),
    IntLit(IntLit),
    UnOp(UnOp),
    BinOp(BinOp),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundStmt {
    pub items: Vec<Ast>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntLit {
    pub val: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnOp {
    pub op: OpUn,
    pub expr: Box<Ast>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpUn {
    Neg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinOp {
    pub op: OpBin,
    pub lhs: Box<Ast>,
    pub rhs: Box<Ast>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpBin {
    Add, // `+`
    Sub, // `-`
    Mul, // `*`
    Div, // `/`
    Mod, // `%`
}
