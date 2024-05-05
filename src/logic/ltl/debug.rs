use super::{ba::BA, gba::GBA, safra::DRA, vwaa::VWAA};

pub fn print_gba(gba: GBA) {
    let renamed_initial = gba.initial;
    let renamed_accept_t = gba.acc_transitions;
    let renamed_trans_f = gba.trans_f;

    println!("gba:");
    println!("Initials: {:?}", renamed_initial);
    for (s, t) in renamed_trans_f {
        for transition in t {
            println!("{} -> {} -> {}", s, transition.props, transition.target);
        }
    }
    if renamed_accept_t.is_empty() {
        println!("No acc transitions!");
    }
    for (_, transitions) in renamed_accept_t {
        for (conj, transition) in transitions {
            println!("{} -> {}", conj, transition);
        }
        println!("----")
    }
}

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
