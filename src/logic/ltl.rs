use crate::common::rename_map;
use crate::input_graph::Node;
use crate::logic::ltl::mdpa::cross_mdp;
use crate::logic::ltl::safra::determinize;
use crate::logic::pctl::{True as Pctl_True, Until as Pctl_Until, AP as Pctl_AP};
use crate::utils::common::Comp;

use self::ba::to_ba;
use super::{Formula, LogicImpl, PctlInfo};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;
use std::{collections::HashSet, fmt::Display};

mod ba;
mod common;
mod gba;
mod mdpa;
mod safra;
mod vwaa;

#[derive(Parser)]
#[grammar = "logic/ltl.pest"]
struct LtlPestParser;

pub struct LtlImpl;

impl LtlImpl {
    fn parse_phi(pair: &Pair<Rule>) -> PhiOp {
        let inner_rules = pair.clone().into_inner().collect::<Vec<Pair<Rule>>>();
        match pair.as_rule() {
            Rule::ap => AP::create(pair.as_str().into()),
            Rule::r#true => True::create(),
            Rule::r#false => False::create(),
            Rule::Phi_alw => {
                let left_phi_raw = inner_rules.first().unwrap();
                let phi = Self::parse_phi(left_phi_raw);
                Release::create(PhiOp::False(False), phi)
            }
            Rule::Phi_and => {
                let left_phi_raw = inner_rules.first().unwrap();
                let left_phi = Self::parse_phi(left_phi_raw);
                let right_phi = Self::parse_phi(inner_rules.get(1).unwrap());
                And::create(left_phi, right_phi)
            }
            Rule::Phi_or => {
                let left_phi_raw = inner_rules.first().unwrap();
                let left_phi = Self::parse_phi(left_phi_raw);
                let right_phi = Self::parse_phi(inner_rules.get(1).unwrap());
                Or::create(left_phi, right_phi)
            }
            Rule::Phi_not => {
                let left_phi_raw = inner_rules.first().unwrap();
                let phi = Self::parse_phi(left_phi_raw);
                phi.negate()
            }
            Rule::Phi_next => {
                let left_phi_raw = inner_rules.first().unwrap();
                let left_phi = Self::parse_phi(left_phi_raw);
                Next::create(left_phi)
            }
            Rule::Phi_until => {
                let left_phi_raw = inner_rules.first().unwrap();
                let left_phi = Self::parse_phi(left_phi_raw);
                let right_phi = Self::parse_phi(inner_rules.get(1).unwrap());
                Until::create(left_phi, right_phi)
            }
            Rule::Phi_release => {
                let left_phi_raw = inner_rules.first().unwrap();
                let left_phi = Self::parse_phi(left_phi_raw);
                let right_phi = Self::parse_phi(inner_rules.get(1).unwrap());
                Release::create(left_phi, right_phi)
            }
            Rule::Phi_ev => {
                let left_phi_raw = inner_rules.first().unwrap();
                let inner_rule = Self::parse_phi(left_phi_raw);
                Until::create(PhiOp::True(True), inner_rule)
            }
            _ => unreachable!(),
        }
    }
}

impl LogicImpl for LtlImpl {
    fn parse(&self, content: &str) -> Box<dyn super::Formula> {
        let phi_content = match self.find_formula(content) {
            None => panic!("Formula must contain 'PHI' exactly once!"),
            Some(c) => c,
        };
        println!("{}", phi_content);
        let pairs: Vec<_> = LtlPestParser::parse(Rule::Main, &phi_content)
            .unwrap()
            .collect();
        let pair = pairs.first().unwrap();
        Box::new(Self::parse_phi(pair))
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
enum PhiOp {
    True(True),
    False(False),
    Not(Not),
    AP(AP),
    Until(Until),
    Next(Next),
    Release(Release),
    And(And),
    Or(Or),
}

impl Formula for PhiOp {
    fn evaluate(&self, pctl_info: &PctlInfo) -> HashSet<NodeIndex> {
        let vwaa = vwaa::to_vwaa(self.clone());
        let gba = gba::to_gba(vwaa);
        let ba = to_ba(gba);
        let dra = determinize(ba);

        let dra_initial = dra.initial.clone();
        let (cross_mdp, aec) = cross_mdp(dra, pctl_info);
        let rename_map = rename_map(&cross_mdp);

        let renamed_mdp = cross_mdp.map(
            |_, node| match node {
                Node::State(_) => Node::State(rename_map.get(node).unwrap().clone()),
                Node::Action(a) => Node::Action(a.clone()),
            },
            |_, e| *e,
        );
        let renamed_initial = rename_map
            .get(&Node::State((pctl_info.initial_marking, dra_initial)))
            .unwrap();
        let mut adapter_ap_map = HashMap::new();
        adapter_ap_map.insert("aec".into(), aec);
        let adapter_pctl_info = PctlInfo {
            initial_marking: *renamed_initial,
            reach_graph: renamed_mdp,
            ap_map: adapter_ap_map,
            max_error: pctl_info.max_error,
        };

        let pctl_until = Pctl_Until {
            prev: Box::new(Pctl_True),
            until: Box::new(Pctl_AP {
                value: "aec".into(),
            }),
        };

        let mut prob_map_min: HashMap<NodeIndex, f64> = HashMap::new();
        let (s_1, s_q) = pctl_until.s1_sq(&adapter_pctl_info, &mut prob_map_min);
        let mut prob_map_max = prob_map_min.clone();
        Pctl_Until::iterate_prob(
            &adapter_pctl_info,
            s_q.clone(),
            &mut prob_map_min,
            s_1.clone(),
            &Comp::Leq,
        );
        Pctl_Until::iterate_prob(&adapter_pctl_info, s_q, &mut prob_map_max, s_1, &Comp::Geq);
        todo!()
    }

    fn fmt(&self) -> String {
        Phi::fmt(self)
    }
}

impl Display for PhiOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&Phi::fmt(self))
    }
}

impl Phi for PhiOp {
    fn fmt(&self) -> String {
        if self.is_temporal() {
            return self.get_value().fmt();
        }
        return format!("({})", self.get_value().fmt());
    }

    fn negate(&self) -> PhiOp {
        self.get_value().negate()
    }

    fn is_temporal(&self) -> bool {
        self.get_value().is_temporal()
    }

    fn get_subformula(&self) -> Vec<PhiOp> {
        self.get_value().get_subformula()
    }

    fn is_atomic(&self) -> bool {
        self.get_value().is_atomic()
    }
}

impl PhiOp {
    fn get_value(&self) -> Box<dyn Phi> {
        match self {
            PhiOp::True(value) => Box::new(value.to_owned()),
            PhiOp::False(value) => Box::new(value.to_owned()),
            PhiOp::Not(value) => Box::new(value.to_owned()),
            PhiOp::AP(value) => Box::new(value.to_owned()),
            PhiOp::Until(value) => Box::new(value.to_owned()),
            PhiOp::Next(value) => Box::new(value.to_owned()),
            PhiOp::Release(value) => Box::new(value.to_owned()),
            PhiOp::And(value) => Box::new(value.to_owned()),
            PhiOp::Or(value) => Box::new(value.to_owned()),
        }
    }

    // Maybe useful for later
    fn _get_name(&self) -> &str {
        match self {
            PhiOp::True(_) => "True",
            PhiOp::False(_) => "False",
            PhiOp::Not(_) => "Not",
            PhiOp::AP(_) => "AP",
            PhiOp::Until(_) => "Until",
            PhiOp::Next(_) => "Next",
            PhiOp::Release(_) => "Release",
            PhiOp::And(_) => "And",
            PhiOp::Or(_) => "Or",
        }
    }
}

trait Phi {
    fn fmt(&self) -> String;
    fn negate(&self) -> PhiOp;
    fn is_temporal(&self) -> bool;
    fn get_subformula(&self) -> Vec<PhiOp>;
    fn is_atomic(&self) -> bool;
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct True;

impl True {
    fn create() -> PhiOp {
        PhiOp::True(True)
    }
}

impl Phi for True {
    fn fmt(&self) -> String {
        "tt".into()
    }

    fn negate(&self) -> PhiOp {
        PhiOp::False(False)
    }

    fn is_temporal(&self) -> bool {
        true
    }

    fn get_subformula(&self) -> Vec<PhiOp> {
        unimplemented!()
    }

    fn is_atomic(&self) -> bool {
        true
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct False;

impl False {
    fn create() -> PhiOp {
        PhiOp::False(False)
    }
}

impl Phi for False {
    fn fmt(&self) -> String {
        "ff".into()
    }

    fn negate(&self) -> PhiOp {
        PhiOp::True(True)
    }

    fn is_temporal(&self) -> bool {
        true
    }

    fn get_subformula(&self) -> Vec<PhiOp> {
        unimplemented!()
    }

    fn is_atomic(&self) -> bool {
        true
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Not {
    ap: AP,
}

impl Not {
    fn create(ap: AP) -> PhiOp {
        PhiOp::Not(Not { ap })
    }
}

impl Phi for Not {
    fn fmt(&self) -> String {
        format!("!{}", self.ap.value)
    }

    fn negate(&self) -> PhiOp {
        PhiOp::AP(AP {
            value: self.ap.value.clone(),
        })
    }

    fn is_temporal(&self) -> bool {
        true
    }

    fn get_subformula(&self) -> Vec<PhiOp> {
        vec![PhiOp::AP(self.ap.clone())]
    }

    fn is_atomic(&self) -> bool {
        true
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct AP {
    value: String,
}

impl AP {
    fn create(value: String) -> PhiOp {
        PhiOp::AP(AP { value })
    }
}

impl Phi for AP {
    fn fmt(&self) -> String {
        self.value.clone()
    }

    fn negate(&self) -> PhiOp {
        Not::create(AP {
            value: self.value.clone(),
        })
    }

    fn is_temporal(&self) -> bool {
        true
    }

    fn get_subformula(&self) -> Vec<PhiOp> {
        unimplemented!()
    }

    fn is_atomic(&self) -> bool {
        true
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Until {
    left_phi: Box<PhiOp>,
    right_phi: Box<PhiOp>,
}

impl Until {
    fn create(left_phi: PhiOp, right_phi: PhiOp) -> PhiOp {
        PhiOp::Until(Until {
            left_phi: Box::new(left_phi),
            right_phi: Box::new(right_phi),
        })
    }
}

impl Phi for Until {
    fn fmt(&self) -> String {
        if matches!(*self.left_phi, PhiOp::True(_)) {
            return format!("F{}", self.right_phi);
        }
        format!(
            "{} U {}",
            Phi::fmt(self.left_phi.as_ref()),
            Phi::fmt(self.right_phi.as_ref())
        )
    }

    fn negate(&self) -> PhiOp {
        let not_left_phi = self.left_phi.negate();
        let not_right_phi = self.right_phi.negate();
        PhiOp::Release(Release {
            left_phi: Box::new(not_left_phi),
            right_phi: Box::new(not_right_phi),
        })
    }

    fn is_temporal(&self) -> bool {
        true
    }

    fn get_subformula(&self) -> Vec<PhiOp> {
        vec![*self.left_phi.clone(), *self.right_phi.clone()]
    }

    fn is_atomic(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Next {
    phi: Box<PhiOp>,
}

impl Next {
    fn create(phi: PhiOp) -> PhiOp {
        PhiOp::Next(Self { phi: Box::new(phi) })
    }
}

impl Phi for Next {
    fn fmt(&self) -> String {
        format!("X {}", Phi::fmt(self.phi.as_ref()))
    }

    fn negate(&self) -> PhiOp {
        PhiOp::Next(Next {
            phi: Box::new(self.phi.negate()),
        })
    }

    fn is_temporal(&self) -> bool {
        true
    }

    fn get_subformula(&self) -> Vec<PhiOp> {
        vec![*self.phi.clone()]
    }

    fn is_atomic(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Release {
    left_phi: Box<PhiOp>,
    right_phi: Box<PhiOp>,
}

impl Release {
    fn create(left_phi: PhiOp, right_phi: PhiOp) -> PhiOp {
        PhiOp::Release(Self {
            left_phi: Box::new(left_phi),
            right_phi: Box::new(right_phi),
        })
    }
}

impl Phi for Release {
    fn fmt(&self) -> String {
        if matches!(*self.left_phi, PhiOp::False(_)) {
            return format!("G{}", self.right_phi);
        }
        format!(
            "{} R {}",
            Phi::fmt(self.left_phi.as_ref()),
            Phi::fmt(self.right_phi.as_ref())
        )
    }

    fn negate(&self) -> PhiOp {
        let not_left_phi = self.left_phi.negate();
        let not_right_phi = self.right_phi.negate();
        PhiOp::Until(Until {
            left_phi: Box::new(not_left_phi),
            right_phi: Box::new(not_right_phi),
        })
    }

    fn is_temporal(&self) -> bool {
        true
    }

    fn get_subformula(&self) -> Vec<PhiOp> {
        vec![*self.left_phi.clone(), *self.right_phi.clone()]
    }

    fn is_atomic(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct And {
    left_phi: Box<PhiOp>,
    right_phi: Box<PhiOp>,
}

impl And {
    fn create(left_phi: PhiOp, right_phi: PhiOp) -> PhiOp {
        if left_phi == right_phi {
            return left_phi;
        }
        match (left_phi.clone(), right_phi.clone()) {
            (PhiOp::True(True), _) => right_phi,
            (_, PhiOp::True(True)) => left_phi,
            (PhiOp::False(False), _) => PhiOp::False(False),
            (_, PhiOp::False(False)) => PhiOp::False(False),
            (_, _) => PhiOp::And(Self {
                left_phi: Box::new(left_phi),
                right_phi: Box::new(right_phi),
            }),
        }
    }

    fn flatten(phi: PhiOp) -> BTreeSet<PhiOp> {
        if let PhiOp::And(and) = phi {
            return Self::flatten(*and.left_phi)
                .union(&Self::flatten(*and.right_phi))
                .cloned()
                .collect();
        }

        let mut result = BTreeSet::new();
        result.insert(phi);
        return result;

        // Maybe I need this later :)
        // if *and.left_phi == phi || *and.right_phi == phi {
        //     return PhiOp::And(and);
        // }
        // if let PhiOp::And(left_and) = *and.left_phi {
        //     return Self::remove_redundancy(left_and, phi);
        // }
        // if let PhiOp::And(right_and) = *and.right_phi {
        //     return Self::remove_redundancy(right_and, phi);
        // }
        // PhiOp::And(Self {
        //     left_phi: Box::new(PhiOp::And(and)),
        //     right_phi: Box::new(phi),
        // })
    }
}

impl Phi for And {
    fn fmt(&self) -> String {
        format!(
            "{} ∧ {}",
            Phi::fmt(self.left_phi.as_ref()),
            Phi::fmt(self.right_phi.as_ref())
        )
    }

    fn negate(&self) -> PhiOp {
        let not_left_phi = self.left_phi.negate();
        let not_right_phi = self.right_phi.negate();
        PhiOp::Or(Or {
            left_phi: Box::new(not_left_phi),
            right_phi: Box::new(not_right_phi),
        })
    }

    fn is_temporal(&self) -> bool {
        false
    }

    fn get_subformula(&self) -> Vec<PhiOp> {
        vec![*self.left_phi.clone(), *self.right_phi.clone()]
    }

    fn is_atomic(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Or {
    left_phi: Box<PhiOp>,
    right_phi: Box<PhiOp>,
}

impl Or {
    fn create(left_phi: PhiOp, right_phi: PhiOp) -> PhiOp {
        match (left_phi.clone(), right_phi.clone()) {
            (PhiOp::True(True), _) => PhiOp::True(True),
            (_, PhiOp::True(True)) => PhiOp::True(True),
            (PhiOp::False(False), _) => right_phi,
            (_, PhiOp::False(False)) => left_phi,
            (_, _) => PhiOp::Or(Self {
                left_phi: Box::new(left_phi),
                right_phi: Box::new(right_phi),
            }),
        }
    }
}

impl Phi for Or {
    fn fmt(&self) -> String {
        format!(
            "{} ∨ {}",
            Phi::fmt(self.left_phi.as_ref()),
            Phi::fmt(self.right_phi.as_ref())
        )
    }

    fn negate(&self) -> PhiOp {
        let not_left_phi = self.left_phi.negate();
        let not_right_phi = self.right_phi.negate();
        PhiOp::And(And {
            left_phi: Box::new(not_left_phi),
            right_phi: Box::new(not_right_phi),
        })
    }

    fn is_temporal(&self) -> bool {
        false
    }

    fn get_subformula(&self) -> Vec<PhiOp> {
        vec![*self.left_phi.clone(), *self.right_phi.clone()]
    }

    fn is_atomic(&self) -> bool {
        false
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Conjuction(BTreeSet<PhiOp>);

impl Display for Conjuction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rep: String = "".into();
        for phi in &self.0 {
            rep.push_str(&Phi::fmt(phi));
            rep.push_str(" ∧ ");
        }
        if let Some(value) = rep.strip_suffix(" ∧ ") {
            rep = value.to_string();
        }
        f.write_str(&rep)
    }
}

impl Conjuction {
    fn _new(left_phi: PhiOp, right_phi: PhiOp) -> Conjuction {
        let mut elements: BTreeSet<PhiOp> = BTreeSet::new();
        elements.insert(left_phi);
        elements.insert(right_phi);
        Conjuction(elements)
    }
    fn _single(phi: PhiOp) -> Conjuction {
        let mut elements: BTreeSet<PhiOp> = BTreeSet::new();
        elements.insert(phi);
        Conjuction(elements)
    }
}
