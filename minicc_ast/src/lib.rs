#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ast {
    pub kind: AstKind,
    pub loc: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstKind {
    CompoundStmt(CompoundStmt),
    Decl(Decl),
    Return(Return),
    Ref(Ref),
    IntLit(IntLit),
    UnOp(UnOp),
    BinOp(BinOp),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundStmt {
    pub items: Vec<Ast>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl {
    pub ident: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Return {
    pub expr: Box<Ast>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref {
    pub ident: String,
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
    Neg,    // `-`
    LogNot, // `!`
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinOp {
    pub op: OpBin,
    pub lhs: Box<Ast>,
    pub rhs: Box<Ast>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpBin {
    Add,   // `+`
    Sub,   // `-`
    Mul,   // `*`
    Div,   // `/`
    Mod,   // `%`
    Lt,    // `<`
    Gt,    // `>`
    Le,    // `<=`
    Ge,    // `>=`
    Eq,    // `==`
    Ne,    // `!=`
    Asign, // `=`
}
