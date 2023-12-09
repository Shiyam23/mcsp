use std::collections::HashSet;
use std::fmt::Display;
use petgraph::graph::NodeIndex;
use crate::mcsp::ModelCheckInfo;
use crate::utils::common::Comp;

pub trait StatePhi {
    fn fmt(&self) -> String;
    fn evaluate<'a>(&'a self, model_check_info: &'a ModelCheckInfo) -> HashSet<NodeIndex>;
}

impl Display for dyn StatePhi {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.fmt())
    }
}

pub trait PathPhi {
    fn fmt(&self) -> String;
    fn evaluate<'a>(
        &'a self,
        model_check_info: &'a ModelCheckInfo,
        comp: &Comp,
        prob_bound: f64,
    ) -> HashSet<NodeIndex>;
}

impl Display for dyn PathPhi {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.fmt())
    }
}

pub struct True;
pub struct NotPhi {
    pub phi: Box<dyn StatePhi>,
}
pub struct Prob {
    pub phi: Box<dyn PathPhi>,
    pub probability: f64,
    pub comp: Comp,
}
pub struct AP {
    pub value: String,
}
pub struct Next {
    pub phi: Box<dyn StatePhi>,
}
pub struct Until {
    pub prev: Box<dyn StatePhi>,
    pub until: Box<dyn StatePhi>,
}
