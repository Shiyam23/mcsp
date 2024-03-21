use super::PhiOp;
use crate::logic::ltl::{And, Phi};
use std::collections::HashSet;

pub fn bar_op(phi: &PhiOp) -> HashSet<PhiOp> {
    let mut set = HashSet::new();
    if phi.is_temporal() {
        set.insert(phi.clone());
        return set;
    }
    if let PhiOp::Or(or) = phi {
        return bar_op(&or.left_phi)
            .union(&bar_op(&or.right_phi))
            .cloned()
            .collect::<HashSet<PhiOp>>()
            .into();
    }
    if let PhiOp::And(and) = phi {
        let mut result_set: HashSet<PhiOp> = HashSet::new();
        let left_set = bar_op(&and.left_phi);
        let right_set = bar_op(&and.right_phi);
        for left_phi in left_set.into_iter() {
            for right_phi in &right_set {
                let conj: PhiOp = And::create(left_phi.clone(), right_phi.clone());
                result_set.insert(conj);
            }
        }
        return result_set;
    }
    unreachable!()
}
