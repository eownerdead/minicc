/// Count of characters from the beginning of the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos(pub usize);

/// Range of source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Span {
    /// Return range from start of `self` to end of `other`.
    ///
    /// ```
    /// use minicc_ast::Span;
    ///
    /// let start = Span { start: 6, end: 8 };
    /// let end = Span { start: 12, end: 23 };
    ///
    /// assert_eq!(start.to(end), Span { start: 6, end: 23 });
    /// ```
    pub fn to(self, other: Self) -> Self {
        Self { start: self.start, end: other.end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ast {
    pub kind: AstKind,
    pub span: Span,
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
