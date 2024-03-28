use super::{
    common::get_rename_map,
    vwaa::{Alphabet, Delta, Transition, Transitions, VWAA},
    And, Conjuction, PhiOp,
};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    fmt::Display,
};

type ConjTransitions = HashSet<ConjTransition>;
pub type SimpleTransitions = HashSet<SimpleTransition>;

pub struct GBA {
    pub initial: HashSet<String>,
    pub trans_f: HashMap<String, SimpleTransitions>,
    pub acc_transitions: HashMap<PhiOp, HashSet<(String, SimpleTransition)>>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ConjTransition {
    props: Alphabet,
    target: Conjuction,
}

impl Display for ConjTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.props, self.target)
    }
}

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

pub fn to_gba(vwaa: VWAA) -> GBA {
    let mut created_states = HashSet::new();
    let mut trans_f: HashMap<Conjuction, ConjTransitions> = HashMap::new();
    let mut pop_queue: VecDeque<Conjuction> = VecDeque::new();
    pop_queue.extend(vwaa.initial.clone());
    created_states.extend(vwaa.initial.clone());
    while let Some(state) = pop_queue.pop_front() {
        if !trans_f.contains_key(&state) {
            let transitions = delta2(&state);
            let cj_transitions: HashSet<ConjTransition> = transitions
                .into_iter()
                .map(|t| ConjTransition {
                    props: t.props,
                    target: Conjuction(And::flatten(t.target)),
                })
                .collect();
            trans_f.insert(state.clone(), cj_transitions.clone());
            for transition in cj_transitions {
                if !created_states.contains(&transition.target) {
                    pop_queue.push_back(transition.target.clone());
                    created_states.insert(transition.target);
                }
            }
        }
    }

    let mut accept_t: HashMap<PhiOp, HashSet<(Conjuction, ConjTransition)>> = HashMap::new();
    for final_state in &vwaa.final_states {
        for (state, transitions) in &trans_f {
            for transition in transitions {
                if !transition.target.0.contains(&final_state)
                    || implies_acc(transition, final_state, &trans_f)
                {
                    match accept_t.entry(final_state.clone()) {
                        Entry::Occupied(mut v) => {
                            v.get_mut().insert((state.clone(), transition.clone()));
                        }
                        Entry::Vacant(v) => {
                            let mut set = HashSet::new();
                            set.insert((state.clone(), transition.clone()));
                            v.insert(set);
                        }
                    };
                }
            }
        }
    }

    prune_transitions(&mut accept_t, &mut trans_f);
    let rename_map = get_rename_map(&trans_f);
    let renamed_trans_f = rename_trans_f(&trans_f, &rename_map);
    let renamed_initial = rename_initial(vwaa.initial, &rename_map);
    let renamed_accept_t = rename_accept_t(accept_t, &rename_map);

    GBA {
        initial: renamed_initial,
        trans_f: renamed_trans_f,
        acc_transitions: renamed_accept_t,
    }
}

fn rename_trans_f(
    trans_f: &HashMap<Conjuction, ConjTransitions>,
    rename_map: &HashMap<&Conjuction, String>,
) -> HashMap<String, SimpleTransitions> {
    trans_f
        .iter()
        .map(|(state, transitions)| {
            let converted_transitions: SimpleTransitions = transitions
                .into_iter()
                .map(|t| SimpleTransition {
                    props: t.props.clone(),
                    target: rename_map.get(&t.target).unwrap().into(),
                })
                .collect();
            (rename_map.get(state).unwrap().into(), converted_transitions)
        })
        .collect()
}

fn rename_initial(
    initial: HashSet<Conjuction>,
    rename_map: &HashMap<&Conjuction, String>,
) -> HashSet<String> {
    initial
        .iter()
        .map(|c| rename_map.get(c).unwrap().into())
        .collect()
}

fn rename_accept_t(
    accept_t: HashMap<PhiOp, HashSet<(Conjuction, ConjTransition)>>,
    rename_map: &HashMap<&Conjuction, String>,
) -> HashMap<PhiOp, HashSet<(String, SimpleTransition)>> {
    accept_t
        .into_iter()
        .map(|(phi, transitions)| {
            let simple_transitions = transitions
                .iter()
                .map(|(c, ct)| {
                    (
                        rename_map.get(c).unwrap().into(),
                        SimpleTransition {
                            props: ct.props.clone(),
                            target: rename_map.get(&ct.target).unwrap().into(),
                        },
                    )
                })
                .collect();
            (phi, simple_transitions)
        })
        .collect()
}

fn prune_transitions(
    accept_t: &mut HashMap<PhiOp, HashSet<(Conjuction, ConjTransition)>>,
    trans_f: &mut HashMap<Conjuction, HashSet<ConjTransition>>,
) {
    for (_, set) in accept_t {
        let set_duplicate = set.clone();
        set.retain(|(s, t)| {
            let delete = set_duplicate.iter().any(|(s2, t2)| {
                t != t2 && s == s2 && t2.props.0.is_subset(&t.props.0) && t2.target.0 == t.target.0
            });
            if delete {
                trans_f.get_mut(s).unwrap().remove(t);
            }
            !delete
        });
    }
}

fn delta2(conj: &Conjuction) -> Transitions {
    conj.0
        .iter()
        .map(|phi| PhiOp::small_delta(phi))
        .reduce(|acc, phi| Transition::cross_op(acc, phi))
        .unwrap()
}

fn implies_acc(
    conj_t: &ConjTransition,
    final_s: &PhiOp,
    trans_f: &HashMap<Conjuction, ConjTransitions>,
) -> bool {
    let transitions = trans_f.get(&conj_t.target).unwrap();
    transitions.iter().any(|t| {
        !t.target.0.contains(final_s)
            && t.target.0.is_subset(&conj_t.target.0)
            && t.props.0.is_subset(&conj_t.props.0)
    })
}