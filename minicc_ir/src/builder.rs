use crate::{Block, Func, Inst, Mod, Var};

#[derive(Debug, Default)]
pub struct Builder {
    pub mod_: Mod,
    pub func: Option<String>,
    pub block: Option<Block>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_to_new_func(&mut self, func: String) {
        self.new_func(func.clone());
        self.move_to_func(func);
    }

    pub fn move_to_new_block(&mut self) -> Block {
        let block = self.new_block();
        self.move_to_block(block);
        block
    }

    pub fn new_func(&mut self, func: String) {
        if self.mod_.funcs.insert(func, Func::new()).is_some() {
            panic!();
        }
    }

    pub fn new_block(&mut self) -> Block {
        self.mod_
            .funcs
            .get_mut(self.func.as_ref().unwrap())
            .unwrap()
            .new_block()
    }

    pub fn new_var(&mut self) -> Var {
        self.mod_.funcs.get_mut(self.func.as_ref().unwrap()).unwrap().new_var()
    }

    pub fn move_to_func(&mut self, func: String) {
        self.func = Some(func);
    }

    pub fn move_to_block(&mut self, block: Block) {
        self.block = Some(block);
    }

    pub fn push_inst(&mut self, inst: Inst) {
        self.mod_
            .funcs
            .get_mut(self.func.as_ref().unwrap())
            .unwrap()
            .blocks
            .get_mut(&self.block.unwrap())
            .unwrap()
            .insts
            .push(inst)
    }
}
