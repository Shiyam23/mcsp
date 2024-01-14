pub mod pnet;

use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use petgraph::graph::DiGraph;

pub type MDP<T> = DiGraph<Node<T>, f64>;
pub type ApMap<T> = HashMap<String, HashSet<T>>;

#[derive(clap::ValueEnum, Clone, Default)]
pub enum InputGraphType {
    #[default]
    Petri,
    DecisionPetri
}

#[derive(PartialEq)]
pub enum Node<T> {
    State(T),
    Action(String)
}

impl<T> Clone for Node<T> where T: Clone{
    fn clone(&self) -> Self {
        match self {
            Node::State(e) => Node::State(e.clone()),
            Node::Action(e) => Node::Action(e.clone())
        }
    }
}

impl<T> Display for Node<T> where T: Display{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::State(t) => write!(f, "State {}", t),
            Node::Action(a) => write!(f, "Action {}", a)
        }
    }
}

impl<T> Debug for Node<T> where T:Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::State(t) => write!(f, "State {:?}", t),
            Node::Action(a) => write!(f, "Action {:?}", a)
        }
    }
}

impl<T> Node<T> where T: Clone {
    pub fn is_state(&self) -> bool {
        match self {
            Node::State(_) => true,
            Node::Action(_) => false
        }
    }

    pub fn get_inner_element(&self) -> Option<T> {
        match self {
            Node::State(e) => Some(e.clone()),
            Node::Action(_) => None
        }
    }
}

pub trait ParseImpl<T: InputGraph> {
    fn parse(content: &str) -> Box<T>;
}

pub trait InputGraph {
    type S: State;
    fn validate_graph(&self);
    fn to_mdp(&self) -> MDP<Self::S>;
    fn get_ap_map(&self) -> &ApMap<Self::S>;
    fn get_init_state(&self) -> &Self::S;
}

pub trait State: Debug + Clone + PartialEq {
    fn test(&self);
}

pub trait GenericInputGraph {}

pub trait GenericMDP {}
pub trait GenericApMap {}
