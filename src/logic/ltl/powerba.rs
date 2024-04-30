use super::{
    ba::BA,
    common::{Alphabet, SimpleTransition},
    PhiOp,
};
use crate::utils::common::powerset;
use std::collections::{BTreeSet, HashMap, HashSet};

#[allow(clippy::upper_case_acronyms)]
pub struct PowerBA {
    pub initials: BTreeSet<String>,
    pub symbols: HashSet<Alphabet>,
    pub transitions: HashMap<String, HashSet<SimpleTransition>>,
    pub finals: BTreeSet<String>,
}

pub fn to_powerba(ba: &BA) -> PowerBA {
    let mut phis: BTreeSet<PhiOp> = BTreeSet::new();
    for symbol in &ba.symbols {
        for phi in &symbol.0 {
            phis.insert(phi.clone());
        }
    }
    let phis_vec: Vec<PhiOp> = phis.into_iter().collect();
    let power_phis: HashSet<Alphabet> = powerset(&phis_vec)
        .into_iter()
        .map(|set| Alphabet(set.into_iter().cloned().collect::<BTreeSet<_>>()))
        .collect();

    let mut new_transition_map: HashMap<String, HashSet<SimpleTransition>> = HashMap::new();
    for (state, transitions) in &ba.transitions {
        let mut new_transitions: HashSet<SimpleTransition> = HashSet::new();
        for transition in transitions {
            power_phis
                .iter()
                .filter(|alph| transition.props.0.iter().all(|prop| alph.0.contains(&prop)))
                .for_each(|alph| {
                    new_transitions.insert(SimpleTransition {
                        props: alph.clone(),
                        target: transition.target.clone(),
                    });
                });
        }
        new_transition_map.insert(state.into(), new_transitions);
    }
    let symbols: HashSet<Alphabet> = power_phis.into();

    PowerBA {
        initials: ba.initials.clone(),
        symbols,
        transitions: new_transition_map,
        finals: ba.finals.clone(),
    }
}
