pub struct Transition {
    pub name: String,
    pub pre: Vec<String>,
    pub pre_markings: Vec<usize>,
    pub post: Vec<String>,
    pub post_markings: Vec<usize>,
}

pub struct AP {
    pub name: String,
    pub markings: Vec<Vec<usize>>,
}

pub fn format_places(n: usize) -> String {
    let mut tmp: String = "P = {".into();
    for i in 1..n {
        tmp += format!("P{}, ", i).as_str();
    }
    tmp += format!("P{}}}\n", n).as_str();
    tmp
}

pub fn format_transitions(transitions: Vec<Transition>) -> String {
    let mut tmp: String = "G = {\n".into();
    for t in &transitions {
        assert_eq!(
            t.pre.len(),
            t.pre_markings.len(),
            "Different number of pres and markings"
        );
        let mut t_str: String = "\t{".into();
        for (pre, m) in t.pre.iter().zip(&t.pre_markings) {
            t_str += format!("P{}({}), ", pre, m).as_str();
        }
        if !t.pre.is_empty() {
            t_str = t_str.strip_suffix(", ").unwrap().into();
        }
        t_str += format!("}} -> {} -> {{", t.name).as_str();
        for (post, m) in t.post.iter().zip(&t.post_markings) {
            t_str += format!("P{}({}), ", post, m).as_str();
        }
        if !t.post.is_empty() {
            t_str = t_str.strip_suffix(", ").unwrap().into();
        }
        t_str += "},\n";
        tmp += t_str.as_str();
    }
    if !transitions.is_empty() {
        tmp = tmp.strip_suffix(",\n").unwrap().into();
    }
    tmp + "\n}\n"
}

pub fn format_controllables(n: usize) -> String {
    let mut tmp: String = "C = {".into();
    for i in 1..n {
        tmp += format!("t{}, ", i).as_str();
    }
    tmp += format!("t{}}}\n", n).as_str();
    tmp
}

pub fn format_initial(initial: Vec<usize>) -> String {
    assert!(!initial.is_empty(), "Initial is empty");
    let mut tmp: String = "M = ".into();
    tmp += format_tuple(initial).as_str();
    tmp + "\n"
}

pub fn format_lambdas(lambdas: Vec<usize>) -> String {
    assert!(!lambdas.is_empty(), "lambdas is empty");
    let mut tmp: String = "L = ".into();
    tmp += format_tuple(lambdas).as_str();
    tmp + "\n"
}

pub fn format_ap(aps: Vec<AP>) -> String {
    let mut tmp: String = "AP = {\n".into();
    for ap in aps {
        tmp += format!("\t({}, {{", ap.name).as_str();
        for m in ap.markings {
            tmp += format!("{}, ", format_tuple(m)).as_str();
        }
        tmp = tmp.strip_suffix(", ").unwrap().into();
        tmp += "}),";
    }
    tmp = tmp.strip_suffix(",").unwrap().into();
    tmp + "\n}\n"
}

pub fn format_tuple(m: Vec<usize>) -> String {
    assert!(!m.is_empty(), "Tuple is empty");
    let mut tmp: String = "(".into();
    for m in &m {
        tmp += format!("{}, ", m).as_str();
    }
    tmp = tmp.strip_suffix(", ").unwrap().into();
    tmp + ")"
}

pub struct PN1;

impl PN1 {
    pub fn get_input(n: usize) -> String {
        assert!(n > 0, "n can not be zero");
        let mut pn = Self::get_pn(n);
        pn += Self::get_transitions(n).as_str();
        pn += Self::get_controllables(n).as_str();
        pn += Self::get_initial(n).as_str();
        pn += Self::get_lambdas(n).as_str();
        pn += Self::get_aps(n).as_str();
        pn += Self::get_formula(n).as_str();
        pn
    }

    fn get_formula(_: usize) -> String {
        "PHI = P((phione) U (phitwo), >= 0.5)".into()
    }

    fn get_pn(n: usize) -> String {
        format_places(2 * n + 1)
    }

    fn get_transitions(n: usize) -> String {
        let mut transitions: Vec<Transition> = Vec::with_capacity(2 * n - 1);

        transitions.push(Transition {
            name: format!("t1"),
            pre: vec!["1".into()],
            pre_markings: vec![1],
            post: vec!["2".into(), "3".into()],
            post_markings: vec![1, 1],
        });
        for i in 1..=n - 1 {
            let k = 2 * i;
            transitions.push(Transition {
                name: format!("t{}", k),
                pre: vec![format!("{}", k)],
                pre_markings: vec![1],
                post: vec![format!("{}", k + 2)],
                post_markings: vec![1],
            });
            transitions.push(Transition {
                name: format!("t{}", k + 1),
                pre: vec![format!("{}", k + 1)],
                pre_markings: vec![1],
                post: vec![format!("{}", k + 3)],
                post_markings: vec![1],
            });
        }
        format_transitions(transitions)
    }

    fn get_controllables(n: usize) -> String {
        format_controllables(2 * n - 1)
    }

    fn get_initial(n: usize) -> String {
        let mut initials: Vec<usize> = Vec::with_capacity(2 * n + 1);
        initials.push(1);
        for _ in 0..2 * n {
            initials.push(0);
        }
        format_initial(initials)
    }

    fn get_lambdas(n: usize) -> String {
        let mut lambdas: Vec<usize> = vec![0; 2 * n - 1];
        lambdas[0] = 1;
        format_lambdas(lambdas)
    }

    fn get_aps(n: usize) -> String {
        let mut aps: Vec<AP> = Vec::new();
        let mut phi_one_markings: Vec<Vec<usize>> = Vec::new();

        let mut init = vec![0; 2 * n + 1];
        init[0] = 1;
        phi_one_markings.push(init);

        let mut first_branch = vec![0; 2 * n + 1];
        first_branch[1] = 1;
        let mut second_branch = vec![0; 2 * n + 1];
        second_branch[2] = 1;

        'outer: while first_branch[0] != 1 {
            let mut second_branch = second_branch.clone();
            while second_branch[1] != 1 {
                let sum = first_branch
                    .iter()
                    .zip(&second_branch)
                    .map(|(f, s)| f + s)
                    .collect::<Vec<usize>>();
                if *sum.last().unwrap() == 1_usize && sum[sum.len() - 2] == 1 {
                    break 'outer;
                }

                phi_one_markings.push(sum.clone());
                second_branch.rotate_right(2);
            }
            first_branch.rotate_right(2);
        }

        aps.push(AP {
            name: "phione".to_string(),
            markings: phi_one_markings,
        });

        let mut end_marking = vec![0; 2 * n + 1];
        let length = end_marking.len();
        end_marking[length - 1] = 1;
        end_marking[length - 2] = 1;

        aps.push(AP {
            name: "phitwo".to_string(),
            markings: vec![end_marking],
        });
        format_ap(aps)
    }
}

pub struct PN2;
