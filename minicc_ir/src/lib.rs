pub mod builder;
pub mod dom;
pub mod gen;
pub mod inst;
pub mod mem2reg;
pub mod pred;
pub mod regalloc;
pub mod to2op;
pub mod sccp;

pub use builder::Builder;
pub use inst::{Block, BlockData, Func, Inst, Mod, OpBin, OpUn, Operand, Var};
pub use regalloc::regalloc;
pub use regalloc::VarLoc;
pub use to2op::to2addr;
pub use typed_index_collections::TiVec;
