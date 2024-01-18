use std::fmt::Display;
use std::process::exit;
use std::str::FromStr;
use log::error;

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
            Comp::Geq => false
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