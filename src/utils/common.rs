use std::fmt::Display;

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
}