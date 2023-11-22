use crate::utils::pnet::{PetriNet, State, Transition};
use rand_distr::{Distribution, Exp};
use std::{process::exit, thread::sleep, time};

pub fn simulate(petri_net: &mut PetriNet) {
    loop {
        // Get activated transitions
        let transitions: &Vec<Transition> = &petri_net.transitions;
        let states: &mut Vec<State> = &mut petri_net.states;
        let activated_transitions = check_activated_transitions(&transitions, &states);

        // If no transition is activated, terminate
        if activated_transitions.is_empty() {
            println!("No transition is activated!");
            exit(0);
        }

        // Get random transition and time
        let (transition, time) = get_rn_transition(activated_transitions);
        println!("{:?}", transition);
        fire_transition(transition, states);

        // Sleep
        println!("{}", time);
        sleep(time::Duration::from_secs_f64(time));
    }
}

fn check_activated_transitions<'a>(
    transitions: &'a Vec<Transition>,
    states: &Vec<State>,
) -> Vec<&'a Transition> {
    return transitions
        .iter()
        .filter(|t| t.pre.iter().all(|&s| states[s].token > 0))
        .collect();
}

fn get_rn_transition<'a>(all_transitions: Vec<&'a Transition>) -> (&'a Transition, f64) {
    let mut fastest_transition: &Transition = &all_transitions[0];
    let mut time: f64 = f64::MAX;
    for transition in all_transitions {
        let exp = Exp::new(transition.fire_rate).unwrap();
        let v = exp.sample(&mut rand::thread_rng());
        println!("Transition: {}, Time: {}", transition.name, v);
        if v < time {
            time = v;
            fastest_transition = transition;
        }
    }
    (fastest_transition, time)
}

fn fire_transition(transition: &Transition, states: &mut Vec<State>) {
    for pre_state_id in transition.pre.iter() {
        match states.get_mut(*pre_state_id) {
            Some(state) => state.token -= 1,
            None => panic!("State with id {} was not found", pre_state_id),
        }
    }
    for succ_state_id in transition.succ.iter() {
        match states.get_mut(*succ_state_id) {
            Some(state) => state.token += 1,
            None => panic!("State with id {} was not found", succ_state_id),
        }
    }
}
