use std::cmp::{Less, Greater, Equal, Ordering};

#[derive(Eq, PartialEq)]
pub struct ReverseOrd<E> {
    pub e: E
}

impl <E> ReverseOrd<E> {
    pub fn new(e: E) -> ReverseOrd<E> {
        ReverseOrd {
            e: e
        }
    }
}

impl <E: Ord> Ord for ReverseOrd<E> {
    fn cmp(&self, other: &ReverseOrd<E>) -> Ordering {
        match self.e.cmp(&other.e) {
            Less => Greater,
            Greater => Less,
            Equal => Equal
        }
    }
}
impl <E: Ord> PartialOrd for ReverseOrd<E>{
    fn partial_cmp(&self, other: &ReverseOrd<E>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
