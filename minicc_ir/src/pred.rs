use std::collections::{BTreeMap, BTreeSet};

use crate::inst::{Block, Func};

pub fn pred_blocks(func: &Func) -> BTreeMap<Block, BTreeSet<Block>> {
    let mut preds = func
        .blocks
        .keys()
        .map(|label| (*label, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();

    for (&label, block) in &func.blocks {
        for succ in block.succ() {
            preds.get_mut(&succ).unwrap().insert(label);
        }
    }

    preds
}
