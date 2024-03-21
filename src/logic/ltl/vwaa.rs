use super::common::bar_op;
use super::{Conjuction, Phi, PhiOp, True};
use crate::logic::ltl::And;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Display;
use std::hash::Hash;

pub type Transitions = HashSet<Transition>;

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
    fn full() -> Alphabet {
        Alphabet(BTreeSet::new())
    }

    fn with_prop(ap: PhiOp) -> Alphabet {
        let mut new_set = BTreeSet::new();
        new_set.insert(ap);
        Alphabet(new_set)
    }
    fn intersection(&self, other: &Alphabet) -> Alphabet {
        let new_set = self.0.union(&other.0).cloned().collect();
        Alphabet(new_set)
    }
}

#[allow(dead_code)]
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct Transition {
    pub props: Alphabet,
    pub target: PhiOp,
}

impl Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.props, self.target)
    }
}

impl Transition {
    fn new(props: Alphabet, target: PhiOp) -> Self {
        Self { props, target }
    }

    pub fn cross_op(left_set: HashSet<Self>, right_set: HashSet<Self>) -> Transitions {
        let mut result_set: Transitions = HashSet::with_capacity(left_set.len() * right_set.len());
        for left_item in left_set {
            for right_item in &right_set {
                let props: Alphabet = left_item.props.intersection(&right_item.props);
                let target: PhiOp =
                    And::create(left_item.target.clone(), right_item.target.clone());
                result_set.insert(Transition { props, target });
            }
        }
        result_set
    }
}

pub struct VWAA {
    pub initial: HashSet<Conjuction>,
    pub delta: HashMap<Conjuction, Transitions>,
    pub final_states: HashSet<PhiOp>,
}

pub trait Delta {
    fn small_delta(phi: &PhiOp) -> Transitions;
    fn big_delta(phi: &PhiOp) -> Transitions;
}

impl Delta for PhiOp {
    fn small_delta(phi: &PhiOp) -> Transitions {
        match phi {
            PhiOp::False(_) => return HashSet::with_capacity(0),
            PhiOp::True(_) => {
                let mut result_set: Transitions = HashSet::with_capacity(1);
                result_set.insert(Transition::new(Alphabet::full(), PhiOp::True(True)));
                return result_set;
            }
            PhiOp::AP(_) | PhiOp::Not(_) => {
                let mut result_set: Transitions = HashSet::with_capacity(1);
                result_set.insert(Transition::new(
                    Alphabet::with_prop(phi.clone()),
                    PhiOp::True(True),
                ));
                return result_set;
            }
            // Commented out because we treat negated propositions as one state
            // PhiOp::Not(_) => {
            //     let mut result_set: Transitions = HashSet::with_capacity(1);
            //     let mut set: HashSet<PhiOp> = all_props.clone();
            //     set.remove(phi);
            //     result_set.insert(Transition::new(set, PhiOp::True(True)));
            //     return result_set;
            // }
            PhiOp::Until(until) => {
                let big_delta_right = Self::big_delta(&until.right_phi);
                let big_delta_left = Self::big_delta(&until.left_phi);
                let mut normal_set = HashSet::with_capacity(1);
                normal_set.insert(Transition::new(
                    Alphabet::full(),
                    PhiOp::Until(until.clone()),
                ));
                let left_cross_normal = Transition::cross_op(big_delta_left, normal_set);
                let mut result_set: Transitions =
                    HashSet::with_capacity(left_cross_normal.len() + big_delta_right.len());
                result_set.extend(left_cross_normal);
                result_set.extend(big_delta_right);
                return result_set;
            }
            PhiOp::Next(next) => {
                let bar_phi = bar_op(&next.phi);
                let mut result_set = HashSet::with_capacity(bar_phi.len());
                for item in bar_phi {
                    let transition = Transition::new(Alphabet::full(), item);
                    result_set.insert(transition);
                }
                return result_set;
            }
            PhiOp::Release(release) => {
                let big_delta_right = Self::big_delta(&release.right_phi);
                let big_delta_left = Self::big_delta(&release.left_phi);
                let mut right_normal_set = HashSet::with_capacity(big_delta_left.len() + 1);
                right_normal_set.insert(Transition::new(
                    Alphabet::full(),
                    PhiOp::Release(release.clone()),
                ));
                right_normal_set.extend(big_delta_left);
                return Transition::cross_op(right_normal_set, big_delta_right);
            }
            _ => unreachable!(),
        }
    }

    fn big_delta(phi: &PhiOp) -> Transitions {
        if phi.is_temporal() {
            return Self::small_delta(phi);
        }
        match phi {
            PhiOp::And(and) => {
                let left_and = Self::big_delta(&and.left_phi);
                let right_and = Self::big_delta(&and.right_phi);
                Transition::cross_op(left_and, right_and)
            }
            PhiOp::Or(or) => {
                let left_or = Self::big_delta(&or.left_phi);
                let right_or = Self::big_delta(&or.right_phi);
                left_or.union(&right_or).cloned().collect()
            }
            _ => unreachable!(),
        }
    }
}

pub fn to_vwaa(phi: PhiOp) -> VWAA {
    let initial: HashSet<Conjuction> = bar_op(&phi)
        .into_iter()
        .map(|phi| Conjuction(And::flatten(phi)))
        .collect();
    let mut trans_f: HashMap<Conjuction, Transitions> = HashMap::new();
    let mut pop_queue: VecDeque<Conjuction> = VecDeque::new();
    pop_queue.extend(initial.clone());
    while let Some(state) = pop_queue.pop_front() {
        for temporal_state in state.0 {
            let transitions: Transitions = PhiOp::big_delta(&temporal_state);
            let flat_state = Conjuction(And::flatten(temporal_state));
            if !trans_f.contains_key(&flat_state) {
                trans_f.insert(flat_state, transitions.clone());
                for transition in transitions {
                    pop_queue.push_back(Conjuction(And::flatten(transition.target)));
                }
            }
        }
    }
    let final_states: HashSet<PhiOp> = trans_f
        .keys()
        .map(|c| c.0.first().unwrap())
        .filter(|&phi| matches!(phi, PhiOp::Until(_)))
        .cloned()
        .collect();
    VWAA {
        initial,
        delta: trans_f,
        final_states,
    }
}
