use super::{
    common::{get_rename_map, Alphabet, SimpleTransition},
    vwaa::{Delta, Transition, Transitions, VWAA},
    And, Conjuction, PhiOp,
};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    fmt::Display,
};

type ConjTransitions = HashSet<ConjTransition>;
pub type SimpleTransitions = HashSet<SimpleTransition>;

#[allow(clippy::upper_case_acronyms)]
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
                if !transition.target.0.contains(final_state)
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
    (trans_f, accept_t) = prune_states(trans_f, accept_t);
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
                .iter()
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
    for set in accept_t.values_mut() {
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

fn prune_states(
    trans_f: HashMap<Conjuction, HashSet<ConjTransition>>,
    accept_t: HashMap<PhiOp, HashSet<(Conjuction, ConjTransition)>>,
) -> (
    HashMap<Conjuction, HashSet<ConjTransition>>,
    HashMap<PhiOp, HashSet<(Conjuction, ConjTransition)>>,
) {
    let mut temp_trans_f: HashMap<Conjuction, HashSet<ConjTransition>> = HashMap::new();
    let mut rename_map: HashMap<Conjuction, Conjuction> = HashMap::new();
    for (state, transitions) in trans_f.clone() {
        let opt_equiv_state = temp_trans_f.iter().find(|(_, ot)| **ot == transitions);
        if let Some((os, _)) = opt_equiv_state {
            rename_map.insert(state, os.clone());
        } else {
            temp_trans_f.insert(state, transitions);
        }
    }
    let new_trans_f = trans_f
        .into_iter()
        .filter(|(k, _)| !rename_map.contains_key(k))
        .map(|(k, transitions)| {
            let mapped_transitions = transitions
                .into_iter()
                .map(|t| {
                    let new_state = rename_map.get(&t.target);
                    if let Some(new_state) = new_state {
                        ConjTransition {
                            props: t.props,
                            target: new_state.clone(),
                        }
                    } else {
                        t
                    }
                })
                .collect();
            (k, mapped_transitions)
        })
        .collect();

    let new_accept_t = accept_t
        .into_iter()
        .map(|(phi, transitions)| {
            let new_transitions = transitions
                .into_iter()
                .filter(|(s, _)| !rename_map.contains_key(s))
                .map(|(s, t)| {
                    let new_transition = match rename_map.get(&t.target) {
                        Some(state) => ConjTransition {
                            props: t.props,
                            target: state.clone(),
                        },
                        None => t,
                    };
                    (s, new_transition)
                })
                .collect();
            (phi, new_transitions)
        })
        .collect();
    (new_trans_f, new_accept_t)
}

fn delta2(conj: &Conjuction) -> Transitions {
    conj.0
        .iter()
        .map(PhiOp::small_delta)
        .reduce(Transition::cross_op)
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
