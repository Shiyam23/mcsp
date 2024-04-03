use super::{
    ba::BA,
    common::{get_rename_map, Alphabet, SimpleTransition},
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fmt::Display,
};

static NODE_START_INDEX: usize = 1;
type TransitionFunction = HashMap<String, HashSet<SimpleTransition>>;

#[allow(clippy::upper_case_acronyms)]
pub struct DRA {
    pub initial: String,
    pub trans_f: HashMap<String, BTreeMap<DRATransition, String>>,
    pub acc: Vec<(HashSet<String>, HashSet<String>)>,
}

impl DRA {
    pub fn delta(&self, state: &String, opt_symbol: Option<&String>) -> String {
        let transition_map = self.trans_f.get(state).unwrap();
        match opt_symbol {
            Some(symbol) => match transition_map.get(&DRATransition::Symbol(symbol.into())) {
                Some(target) => target,
                None => transition_map.get(&DRATransition::Others).unwrap(),
            },
            None => transition_map.get(&DRATransition::Others).unwrap(),
        }
        .into()
    }
}

#[derive(PartialOrd, Ord, Eq, PartialEq, Debug)]
pub enum DRATransition {
    Symbol(String),
    Others,
}

#[derive(Clone, PartialEq)]
struct Transition {
    props: Alphabet,
    target: SafraTree,
}

#[derive(Hash, PartialEq, Eq, Clone)]
struct SafraNode {
    index: usize,
    labels: BTreeSet<String>,
    children: Vec<SafraNode>,
    marked: bool,
}

impl Display for SafraNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--------------------")?;
        writeln!(
            f,
            "Node: {:?}, Index: {}, Marked: {}",
            self.labels, self.index, self.marked
        )?;
        for child in &self.children {
            writeln!(f, "{}", child)?;
        }
        writeln!(f, "--------------------")
    }
}

impl SafraNode {
    fn with_labels(labels: BTreeSet<String>, index: usize) -> Self {
        SafraNode {
            index,
            labels,
            children: Vec::new(),
            marked: false,
        }
    }

    fn get_max_id(&self) -> usize {
        self.children
            .iter()
            .map(|c| c.get_max_id())
            .max()
            .unwrap_or(self.index)
    }

    fn remove_symbol(&mut self, rem_labels: &HashSet<&String>) {
        self.labels.retain(|l| !rem_labels.contains(l));
    }

    fn merge_horizontal(&mut self) {
        let length = self.children.len();
        if length < 2 {
            return;
        }
        for i in 0..length - 1 {
            for j in (i + 1)..length {
                let older_child = self.children.get(i).unwrap().clone();
                let younger_child = self.children.get_mut(j).unwrap();
                let younger_labels = younger_child.labels.clone();
                let intersection = older_child
                    .labels
                    .intersection(&younger_labels)
                    .collect::<HashSet<_>>();

                // Delete the labels of the younger child and of his children
                younger_child.remove_symbol(&intersection);
                for grand_children in &mut younger_child.children {
                    grand_children.remove_symbol(&intersection);
                }
            }
        }
    }

    fn merge_vertical(&mut self) {
        if self.children.is_empty() {
            return;
        }
        let labels_united = self
            .children
            .iter()
            .flat_map(|sn| sn.labels.clone())
            .collect::<BTreeSet<_>>();
        if self.labels == labels_united {
            self.children.clear();
            self.marked = true;
            return;
        }
        self.children.iter_mut().for_each(|n| n.merge_vertical());
    }

    fn remove_mark(&mut self) {
        self.marked = false;
        for child in &mut self.children {
            child.remove_mark();
        }
    }

    fn branch_finals(&mut self, acc: &BTreeSet<String>) {
        for child in &mut self.children {
            child.branch_finals(acc);
        }
        let finals = self
            .labels
            .intersection(acc)
            .cloned()
            .collect::<BTreeSet<_>>();
        let new_id = self.get_max_id() + 1;
        let new_node = SafraNode::with_labels(finals, new_id);
        self.children.push(new_node);
    }

    fn powerset(&mut self, trans_f: &TransitionFunction, symbol: &Alphabet) {
        let mut new_labels = BTreeSet::new();
        for label in &self.labels {
            let target = trans_f.get(label).unwrap();
            target
                .iter()
                .filter(|t| t.props == *symbol || t.props.0.is_empty())
                .map(|t| t.target.clone())
                .for_each(|l| {
                    new_labels.insert(l);
                });
        }
        self.labels = new_labels;
        self.children
            .iter_mut()
            .for_each(|c| c.powerset(trans_f, symbol));
    }

    fn remove_empty_nodes(&mut self) {
        self.children.retain(|c| !c.labels.is_empty());
        self.children
            .iter_mut()
            .for_each(|c| c.remove_empty_nodes());
    }

    fn get_by_id(&self, id: usize) -> Option<&SafraNode> {
        if self.index == id {
            return Some(self);
        }
        self.children
            .iter()
            .map(|c| c.get_by_id(id))
            .find(|node| node.is_some())?
    }
}

#[derive(Hash, PartialEq, Eq, Clone)]
struct SafraTree {
    root: SafraNode,
}

impl Display for SafraTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root)
    }
}

impl SafraTree {
    fn with_root(label: BTreeSet<String>) -> SafraTree {
        let root_node = SafraNode::with_labels(label, NODE_START_INDEX);
        SafraTree { root: root_node }
    }

    fn succ_tree(
        &self,
        old_trans_f: &TransitionFunction,
        acc: &BTreeSet<String>,
        symbols: &HashSet<Alphabet>,
    ) -> Vec<Transition> {
        // Step 1 and 2 are independent from the transition
        let mut new_tree = self.clone();
        new_tree.root.remove_mark();
        new_tree.root.branch_finals(acc);

        let mut succ_trees = Vec::new();

        // Step 3, 4, 5 and 6 for every transition
        for symbol in symbols {
            let mut new_tree_symbol = new_tree.clone();
            new_tree_symbol.root.powerset(old_trans_f, symbol);
            new_tree_symbol.root.merge_horizontal();
            new_tree_symbol.root.remove_empty_nodes();
            new_tree_symbol.root.merge_vertical();
            let transition = Transition {
                props: symbol.clone(),
                target: new_tree_symbol,
            };
            succ_trees.push(transition);
        }
        succ_trees
    }
}

pub fn determinize(ba: BA) -> DRA {
    let mut trans_f = HashMap::new();
    let mut pop_queue: VecDeque<SafraTree> = VecDeque::new();
    let initial_tree = SafraTree::with_root(ba.initials);
    pop_queue.push_back(initial_tree.clone());
    while let Some(safra_tree) = pop_queue.pop_front() {
        let succ = safra_tree
            .clone()
            .succ_tree(&ba.transitions, &ba.finals, &ba.symbols);
        trans_f.insert(safra_tree.clone(), succ.clone());
        succ.into_iter().for_each(|t| {
            if !(pop_queue.contains(&t.target) || trans_f.contains_key(&t.target)) {
                pop_queue.push_back(t.target);
            }
        });
    }

    // Rename all safra trees to simple state names
    let rename_map = get_rename_map(&trans_f);

    // Get accepting tuples
    let max_node = trans_f.keys().map(|st| st.root.get_max_id()).max().unwrap();
    let mut acc = Vec::with_capacity(max_node);

    for i in NODE_START_INDEX..=max_node {
        let mut non_exist = HashSet::new();
        let mut marked = HashSet::new();
        for state in trans_f.keys() {
            if let Some(node) = state.root.get_by_id(i) {
                if node.marked {
                    marked.insert(rename_map.get(state).unwrap().into());
                }
            } else {
                non_exist.insert(rename_map.get(state).unwrap().into());
            }
        }
        acc.push((non_exist, marked));
    }

    let mut renamed_trans_f: HashMap<String, BTreeMap<DRATransition, String>> = HashMap::new();
    for (state, transitions) in &trans_f {
        let state_name: String = rename_map.get(&state).unwrap().into();
        renamed_trans_f.insert(state_name.clone(), BTreeMap::new());
        for transition in transitions {
            let phi = transition.props.0.first();
            let dra_transition = match phi {
                Some(phi) => match phi {
                    super::PhiOp::AP(ap) => DRATransition::Symbol(ap.value.clone()),
                    _ => unreachable!(),
                },
                None => DRATransition::Others,
            };
            let new_target = rename_map.get(&transition.target).unwrap().into();
            renamed_trans_f
                .get_mut(&state_name)
                .unwrap()
                .insert(dra_transition, new_target);
        }
    }

    let renamed_initial: String = rename_map.get(&initial_tree).unwrap().into();

    DRA {
        initial: renamed_initial,
        trans_f: renamed_trans_f,
        acc,
    }
}
