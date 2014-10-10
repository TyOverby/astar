use super::reverse_ord::ReverseOrd as Rord;
use std::cmp::Ordering;

#[deriving(Eq, PartialEq)]
pub struct NodeWrapper<N, C> {
    pub n: N,
    pub c: Rord<C>
}

impl <N, C> NodeWrapper<N,C> {
    pub fn new(n: N, c: C) -> NodeWrapper<N, C> {
        NodeWrapper {
            n: n,
            c: Rord::new(c)
        }
    }
}

impl <N: Eq, C: Ord> Ord for NodeWrapper<N, C> {
    fn cmp(&self, other: &NodeWrapper<N, C>) -> Ordering {
        self.c.cmp(&other.c)
    }
}

impl <N: PartialEq, C: Ord> PartialOrd for NodeWrapper<N, C> {
    fn partial_cmp(&self, other: &NodeWrapper<N, C>) -> Option<Ordering> {
        self.c.partial_cmp(&other.c)
    }
}

