use std::cmp::{Ordering};
use super::node::Node;

/// A wrapper around a reference to a node.
/// This is only used when placed inside the priority queue.
pub struct WrappedNode<'a, N: 'a, C:'a> {
    pub node: &'a Node<'a, N, C>
}

impl <'a, N, C> WrappedNode<'a, N, C> {
    pub fn new(node: &'a Node<'a, N, C>) -> WrappedNode<'a, N, C> {
        WrappedNode {
            node: node
        }
    }
}

// Only compare heuristic costs.
impl <'a, N, C> PartialEq for WrappedNode<'a, N, C>
where C: PartialEq {
    fn eq(&self, other: &WrappedNode<'a, N, C>) -> bool {
        let this_cost = self.node.cost_with_heuristic.borrow();
        let that_cost = other.node.cost_with_heuristic.borrow();
        *this_cost == *that_cost
    }
}

impl <'a, N, C> Eq for WrappedNode<'a, N, C> where C: PartialEq { }

impl <'a, N, C> PartialOrd for WrappedNode<'a, N, C>
where C: PartialOrd {
    fn partial_cmp(&self, other: &WrappedNode<'a, N, C>) -> Option<Ordering> {
        let this_cost = self.node.cost_with_heuristic.borrow();
        let that_cost = other.node.cost_with_heuristic.borrow();
        // Match backwards so that the priority queue
        // prioritizes minimal values.
        (*that_cost).partial_cmp(&*this_cost)
    }
}

impl <'a, N, C> Ord for WrappedNode<'a, N, C>
where C: PartialOrd {
    fn cmp(&self, other: &WrappedNode<'a, N, C>) -> Ordering {
        let this_cost = self.node.cost_with_heuristic.borrow();
        let that_cost = other.node.cost_with_heuristic.borrow();
        // Match backwards so that the priority queue
        // prioritizes minimal values.
        match (*that_cost).partial_cmp(&*this_cost) {
            None => Ordering::Equal,
            Some(x) => x
        }
    }
}
