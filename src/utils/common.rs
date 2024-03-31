use log::error;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::process::exit;
use std::str::FromStr;

#[derive(PartialEq)]
pub enum Comp {
    Less,
    Leq,
    Greater,
    Geq,
}

impl Display for Comp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char = match self {
            Comp::Less => "<",
            Comp::Leq => "<=",
            Comp::Greater => ">",
            Comp::Geq => ">=",
        };
        write!(f, "{}", char)
    }
}
impl Comp {
    pub fn evaluate<T: PartialOrd>(&self, first: T, second: T) -> bool {
        match self {
            Comp::Less => first < second,
            Comp::Leq => first <= second,
            Comp::Greater => first > second,
            Comp::Geq => first >= second,
        }
    }

    pub fn is_upper_bound(&self) -> bool {
        match self {
            Comp::Less => true,
            Comp::Leq => true,
            Comp::Greater => false,
            Comp::Geq => false,
        }
    }
}

pub trait ParseOrQuit {
    fn parse_or_quit<T: FromStr>(&self, type_name: &str) -> T;
}
impl ParseOrQuit for &str {
    fn parse_or_quit<T: FromStr>(&self, type_name: &str) -> T {
        let test = self.parse::<T>();
        match test {
            Ok(value) => value,
            Err(_) => {
                error!("{} is not valid {}! Terminating", self, type_name);
                exit(0);
            }
        }
    }
}

// A utility function to determine the power set of a set (here given as a Vec)
// This code fragment was taken from StackOverflow. My special thanks goes to erip!
// Title: powerset
// Author: erip (https://stackoverflow.com/users/2883245/erip)
// Date: Nov 21, 2016
// Type: Source Code
// Availability: https://stackoverflow.com/a/40719103/16463801
// License: https://creativecommons.org/licenses/by-sa/3.0/
pub fn powerset<T>(s: &[T]) -> Vec<Vec<&T>> {
    (0..2usize.pow(s.len() as u32))
        .map(|i| {
            s.iter()
                .enumerate()
                .filter(|&(t, _)| (i >> t) % 2 == 1)
                .map(|(_, element)| element)
                .collect()
        })
        .collect()
}

#[allow(dead_code)]
pub fn reverse_map<K, V>(map: &HashMap<K, HashSet<V>>) -> HashMap<&V, HashSet<&K>>
where
    V: Hash + Eq,
    K: Hash + Eq,
{
    let mut reversed_map = HashMap::new();
    for (k, set) in map {
        for v in set {
            if None == reversed_map.get(v) {
                reversed_map.insert(v, HashSet::new());
            }
            reversed_map.get_mut(v).unwrap().insert(k);
        }
    }
    reversed_map
}
