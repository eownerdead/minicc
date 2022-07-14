#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ast {
    pub kind: AstKind,
    pub loc: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstKind {
    CompoundStmt(CompoundStmt),
    If(If),
    For(For),
    Decl(Decl),
    Return(Return),
    Dbg(Dbg),
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
pub struct If {
    pub cond: Box<Ast>,
    pub then: Box<Ast>,
    pub else_: Option<Box<Ast>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct For {
    pub init: Option<Box<Ast>>,
    pub cond: Option<Box<Ast>>,
    pub inc: Option<Box<Ast>>,
    pub body: Box<Ast>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl {
    pub ident: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Return {
    pub expr: Box<Ast>,
}

// Ad hoc printf until implementing function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dbg {
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
