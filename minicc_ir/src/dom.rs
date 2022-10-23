use std::collections::{BTreeMap, BTreeSet, HashSet};

use typed_index_collections::TiVec;

use crate::inst::{Block, Func};

pub fn dom(
    func: &Func,
    preds: &BTreeMap<Block, BTreeSet<Block>>,
) -> BTreeMap<Block, BTreeSet<Block>> {
    let mut doms = BTreeMap::new();
    for &label in func.blocks.keys() {
        if label == Block(0) {
            doms.insert(label, BTreeSet::from([label]));
        } else {
            doms.insert(label, func.blocks.keys().copied().collect());
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for label in func.blocks.keys() {
            let pred_doms = preds[label]
                .iter()
                .map(|i| doms[i].clone())
                .reduce(|acc, i| acc.intersection(&i).cloned().collect())
                .unwrap_or_default();
            if !pred_doms.is_empty() {
                let mut new_doms = pred_doms.clone();
                new_doms.insert(*label);
                if Some(&new_doms) != doms.get(label) {
                    changed = true;
                    doms.insert(*label, new_doms);
                }
            }
        }
    }

    doms
}

pub fn dom_frontier(
    func: &Func,
    preds: &BTreeMap<Block, BTreeSet<Block>>,
    doms: &BTreeMap<Block, BTreeSet<Block>>,
) -> BTreeMap<Block, BTreeSet<Block>> {
    let mut dfs = BTreeMap::new();

    for &l in func.blocks.keys() {
        let mut df1 = HashSet::new();
        let mut df2 = HashSet::new();
        for &label in func.blocks.keys() {
            for pred in &preds[&label] {
                if doms[pred].contains(&l) {
                    df1.insert(label);
                }
            }
            if !(doms[&label].contains(&l) && (l != label)) {
                df2.insert(label);
            }
        }
        dfs.insert(l, df1.intersection(&df2).copied().collect::<BTreeSet<_>>());
    }
    dfs
}
