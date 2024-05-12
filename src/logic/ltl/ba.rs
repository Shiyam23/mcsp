use super::{
    common::{get_rename_map, Alphabet, SimpleTransition},
    gba::{SimpleTransitions, GBA},
};
use std::{
    cmp::Reverse,
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
};

type State = (String, usize);

#[derive(Hash, Eq, PartialEq, Clone)]
struct Transition {
    props: Alphabet,
    target: State,
}

#[allow(clippy::upper_case_acronyms)]
pub struct BA {
    pub initials: BTreeSet<String>,
    pub symbols: HashSet<Alphabet>,
    pub transitions: HashMap<String, HashSet<SimpleTransition>>,
    pub finals: BTreeSet<String>,
}

pub fn to_ba(gba: GBA) -> BA {
    let initials: HashSet<State> = gba.initial.into_iter().map(|c| (c, 0)).collect();
    let mut symbols = HashSet::new();
    let mut pop_queue: VecDeque<State> = VecDeque::new();
    let mut trans_f: HashMap<State, HashSet<Transition>> = HashMap::new();
    let mut acc_transitions: Vec<HashSet<(String, SimpleTransition)>> =
        gba.acc_transitions.into_values().collect();
    acc_transitions.sort_by_key(|s1| Reverse(s1.len()));
    pop_queue.extend(initials);
    while let Some(state) = pop_queue.pop_front() {
        let transitions = delta(&state, &gba.trans_f, &acc_transitions);
        trans_f.insert(state.clone(), transitions);
        for transition in trans_f.get(&state).unwrap() {
            let target = transition.target.clone();
            symbols.insert(transition.props.clone());
            if !trans_f.contains_key(&target) && !pop_queue.contains(&target) {
                pop_queue.push_back(target);
            }
        }
    }

    trans_f = prune_states(trans_f, acc_transitions.len());
    let rename_map = get_rename_map(&trans_f);
    let initials = trans_f
        .keys()
        .filter(|(_, index)| *index == 0)
        .map(|state| rename_map.get(state).unwrap().clone())
        .collect::<BTreeSet<_>>();
    let finals = trans_f
        .keys()
        .filter(|(_, index)| *index == acc_transitions.len())
        .map(|state| rename_map.get(state).unwrap().clone())
        .collect::<BTreeSet<_>>();
    let renamed_trans_f: HashMap<String, HashSet<SimpleTransition>> = trans_f
        .iter()
        .map(|(state, transitions)| {
            let renamed_transitions: HashSet<SimpleTransition> = transitions
                .iter()
                .map(|transition| SimpleTransition {
                    props: transition.props.clone(),
                    target: rename_map.get(&transition.target).unwrap().clone(),
                })
                .collect();
            let renamed_state = rename_map.get(&state).unwrap().clone();
            (renamed_state, renamed_transitions)
        })
        .collect();

    BA {
        initials,
        symbols,
        transitions: renamed_trans_f,
        finals,
    }
}

fn delta(
    state: &State,
    old_delta: &HashMap<String, SimpleTransitions>,
    acc_t: &Vec<HashSet<(String, SimpleTransition)>>,
) -> HashSet<Transition> {
    let mut result: HashSet<Transition> = HashSet::new();
    let delta_state = old_delta.get(&state.0).unwrap();
    for t in delta_state {
        let next_j = next(acc_t, state.1, &(state.0.clone(), t.clone()));
        let tran = Transition {
            props: t.props.clone(),
            target: (t.target.clone(), next_j),
        };
        result.insert(tran);
    }
    prune_transitions(&mut result);
    result
}

fn next(
    acc_t: &Vec<HashSet<(String, SimpleTransition)>>,
    j: usize,
    transition: &(String, SimpleTransition),
) -> usize {
    let start_index = match j == acc_t.len() {
        true => 0,
        false => j,
    };

    return (start_index..=acc_t.len())
        .filter(|&i| {
            ((start_index + 1)..=i).all(|k| acc_t.get(k - 1).unwrap().contains(transition))
        })
        .max()
        .unwrap_or(start_index);
}

fn prune_transitions(transitions: &mut HashSet<Transition>) {
    let copy = transitions.clone();
    transitions.retain(|t| {
        !copy.iter().any(|ot| {
            t.props != ot.props && t.props.0.is_subset(&ot.props.0) && t.target == ot.target
        })
    })
}

fn prune_states(
    mut trans_f: HashMap<State, HashSet<Transition>>,
    r: usize,
) -> HashMap<State, HashSet<Transition>> {
    let mut temp_trans_f: HashMap<State, HashSet<Transition>> = HashMap::new();
    let mut rename_map: HashMap<State, State> = HashMap::new();
    for (state, transitions) in trans_f.clone() {
        let opt_equiv_state = temp_trans_f
            .iter()
            .find(|(os, ot)| **ot == transitions && ((os.1 == r) == (state.1 == r)));
        if let Some((os, _)) = opt_equiv_state {
            rename_map.insert(state, os.clone());
        } else {
            temp_trans_f.insert(state, transitions);
        }
    }
    trans_f.retain(|k, _| temp_trans_f.contains_key(k));
    trans_f
        .into_iter()
        .map(|(k, transitions)| {
            let mapped_transitions = transitions
                .into_iter()
                .map(|t| {
                    let new_state = rename_map.get(&t.target);
                    if let Some(new_state) = new_state {
                        Transition {
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
        .collect()
}
