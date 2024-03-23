use super::{
    gba::{SimpleTransition, SimpleTransitions, GBA},
    vwaa::Alphabet,
};
use std::collections::{HashMap, HashSet, VecDeque};

type State = (String, usize);

#[derive(Hash, Eq, PartialEq, Clone)]
struct Transition {
    props: Alphabet,
    target: State,
}

pub fn to_ba(gba: GBA) {
    let initials: HashSet<State> = gba.initial.into_iter().map(|c| (c, 0)).collect();

    let mut pop_queue: VecDeque<State> = VecDeque::new();
    let mut trans_f: HashMap<State, HashSet<Transition>> = HashMap::new();
    let mut acc_transitions: Vec<HashSet<(String, SimpleTransition)>> =
        gba.acc_transitions.into_iter().map(|(_, v)| v).collect();
    acc_transitions.sort_by(|s2, s1| s1.len().cmp(&s2.len()));
    pop_queue.extend(initials);
    while let Some(state) = pop_queue.pop_front() {
        let transitions = delta(&state, &gba.trans_f, &acc_transitions);
        trans_f.insert(state.clone(), transitions);
        for transition in trans_f.get(&state).unwrap() {
            let target = transition.target.clone();
            if !trans_f.contains_key(&target) && !pop_queue.contains(&target) {
                pop_queue.push_back(target);
            }
        }
    }
}

fn delta(
    state: &State,
    old_delta: &HashMap<String, SimpleTransitions>,
    acc_t: &Vec<HashSet<(String, SimpleTransition)>>,
) -> HashSet<Transition> {
    old_delta
        .get(&state.0)
        .unwrap()
        .iter()
        .map(|t| {
            let next_j = next(acc_t, state.1, &(state.0.clone(), t.clone()));
            let tran = Transition {
                props: t.props.clone(),
                target: (t.target.clone(), next_j),
            };
            tran
        })
        .inspect(|f| {
            println!(
                "({}, {}) -> {} -> ({}, {})",
                state.0, state.1, f.props, f.target.0, f.target.1
            )
        })
        .collect()
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
        .into_iter()
        .filter(|&i| {
            ((start_index + 1)..=i)
                .into_iter()
                .all(|k| acc_t.get(k - 1).unwrap().contains(transition))
        })
        .max()
        .unwrap_or(start_index);
}
