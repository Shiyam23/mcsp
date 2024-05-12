use super::{ba::BA, gba::GBA, powerba::PowerBA, safra::DRA, vwaa::VWAA};

#[allow(dead_code)]
pub fn print_gba(gba: &GBA) {
    let renamed_initial = &gba.initial;
    let renamed_accept_t = &gba.acc_transitions;
    let renamed_trans_f = &gba.trans_f;

    println!("gba:");
    println!("Initials: {:?}", renamed_initial);
    println!("Transitions:");
    for (s, t) in renamed_trans_f {
        for transition in t {
            println!("{} -> {} -> {}", s, transition.props, transition.target);
        }
    }
    if renamed_accept_t.is_empty() {
        println!("No acc transitions!");
    }
    println!("Acc_t:");
    for (_, transitions) in renamed_accept_t {
        for (conj, transition) in transitions {
            println!("{} -> {}", conj, transition);
        }
        println!("----")
    }
}

#[allow(dead_code)]
pub fn print_ba(ba: &BA) {
    println!("ba:");
    println!("Initials: {:?}", ba.initials);
    for (s, t) in &ba.transitions {
        for transition in t {
            println!("{} -> {} -> {}", s, transition.props, transition.target);
        }
    }
    println!("Finals: {:?}", ba.finals);
    println!("-------------------------------");
}

#[allow(dead_code)]
pub fn print_dra(dra: &DRA) {
    println!("Initial: {}", dra.initial);
    println!("TransitionFunction:");
    for (state, map) in &dra.trans_f {
        for (alphabet, target) in map {
            println!("{} -> {} -> {}", state, alphabet, target);
        }
    }
    println!("Acc:");
    for (l, k) in &dra.acc {
        println!("{:?}, {:?}", l, k);
    }
}

#[allow(dead_code)]
pub fn print_vwaa(vwaa: &VWAA) {
    println!("Vwaa:");
    println!("initial: {:?}", vwaa.initial);
    for (conj, transitions) in &vwaa.delta {
        for transition in transitions {
            println!("{} -> {} -> {}", conj, transition.props, transition.target);
        }
    }
    println!("Finals: {:?}", vwaa.final_states);
    println!("-------------------------");
}

#[allow(dead_code)]
pub fn print_powerba(power_ba: &PowerBA) {
    println!("Initials: {:?}", power_ba.initials);
    println!("-------------------");
    println!("Trans_f:");
    for (state, transitions) in &power_ba.transitions {
        for transition in transitions {
            println!("{} -> {} -> {}", state, transition.props, transition.target);
        }
    }
    println!("Finals: {:?}", power_ba.finals);
    println!("-------------------------------");
}
