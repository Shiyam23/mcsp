use super::PhiOp;
use crate::logic::ltl::{And, Phi};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct SimpleTransition {
    pub props: Alphabet,
    pub target: String,
}

impl Display for SimpleTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.props, self.target)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Alphabet(pub BTreeSet<PhiOp>);

impl Display for Alphabet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rep: String = "{".into();
        for ap in &self.0 {
            rep.push_str(&Phi::fmt(ap));
            rep.push_str(", ");
        }
        if let Some(value) = rep.strip_suffix(", ") {
            rep = value.to_string();
        }
        rep.push('}');
        f.write_str(&rep)
    }
}

impl Alphabet {
    pub fn full() -> Alphabet {
        Alphabet(BTreeSet::new())
    }

    pub fn with_prop(ap: PhiOp) -> Alphabet {
        let mut new_set = BTreeSet::new();
        new_set.insert(ap);
        Alphabet(new_set)
    }
    pub fn intersection(&self, other: &Alphabet) -> Alphabet {
        let new_set = self.0.union(&other.0).cloned().collect();
        Alphabet(new_set)
    }
}

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
            .collect::<HashSet<PhiOp>>();
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
            index += 1;
        }
    }
    conj_to_index
}
