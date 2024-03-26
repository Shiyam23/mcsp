use super::PhiOp;
use crate::logic::ltl::{And, Phi};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

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

pub fn get_rename_map<S, T>(trans_f: &HashMap<S, T>) -> HashMap<&S, String>
where
    S: PartialEq + Eq + Hash,
    T: PartialEq,
{
    let mut conj_to_index: HashMap<&S, String> = HashMap::with_capacity(trans_f.len());
    let mut half_simple_trans_f: HashMap<String, &T> = HashMap::with_capacity(trans_f.len());
    let mut index = 1;
    for (conj, transitions) in trans_f {
        let redundant_state = half_simple_trans_f
            .iter()
            .find(|(_, delta)| transitions == **delta)
            .map(|(state, _)| state);
        if let Some(state) = redundant_state {
            conj_to_index.insert(conj, state.to_string());
        } else {
            half_simple_trans_f.insert(index.to_string(), transitions);
            conj_to_index.insert(conj, index.to_string());
            index = index + 1;
        }
    }
    return conj_to_index;
}
