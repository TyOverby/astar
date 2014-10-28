extern crate arena;

use std::hash::Hash;
use std::num::Zero;

#[cfg(test)]
mod test;

mod node {
    use std::cell::RefCell;
    use std::hash::Hash;
    use std::hash::sip::SipState;
    use std::cmp::Equiv;

    /// DumbNode is used so that we can look up entire Node instances
    /// out of a hashmap even if we only have the state.  This feels
    /// like an awful hack, and it probably is.
    #[deriving(Hash)]
    pub struct DumbNode<'b, N: 'b>(pub &'b N);

    impl <'a, 'b, N: PartialEq, C> Equiv<&'a Node<'a, N, C>> for DumbNode<'b, N> {
        fn equiv(&self, other: &&Node<'a, N, C>) -> bool {
            let DumbNode(x) = *self;
            x.eq(other.state.borrow().deref().as_ref().unwrap())
        }
    }

    /// The main node structure.  It is effectively a wrapper around a
    /// state with some metadata associated with it.
    pub struct Node<'a, N: 'a, C: 'a> {
        /// The user-provided state.
        pub state: RefCell<Option<N>>,
        /// The node wrapping around the state that this
        /// node came from.  Used for backwards traversals.
        pub parent: RefCell<Option<&'a Node<'a, N, C>>>,
        /// If the node is currently in the open set.
        /// A node must either be open or closed.
        pub open: RefCell<bool>,
        /// The cost to get to this node.
        pub cost: RefCell<C>,
        /// The cost to get to this node plus the
        /// expected cost to get to the goal.
        pub cost_with_heuristic: RefCell<C>
    }

    // Only hash the state.
    impl <'a, N, C> Hash for Node<'a, N, C>
    where N: Hash {
        fn hash(&self, hash_state: &mut SipState) {
            self.state.borrow().as_ref().unwrap().hash(hash_state);
        }
    }

    // Only compare the state.
    impl <'a, N, C> PartialEq for Node<'a, N, C>
    where N: PartialEq {
        fn eq(&self, other: &Node<'a, N, C>) -> bool {
            self.state.borrow().as_ref().unwrap().eq(
                other.state.borrow().as_ref().unwrap())
        }
    }

    impl <'a, N, C> Eq for Node<'a, N, C> where N: PartialEq { }
}

mod wrap {
    use std::cmp::{Ordering, Equal};
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
                None => Equal,
                Some(x) => x
            }
        }
    }
}

mod state {
    use arena::TypedArena;
    use super::node::{Node, DumbNode};
    use super::wrap::WrappedNode as Wnode;
    use std::collections::{PriorityQueue, HashMap};
    use std::cell::RefCell;
    use std::hash::Hash;

    /// The place where all of the information about the
    /// current A* process is held.
    /// N is the type of the user-provided state, and
    /// C is the type of the user-provided cost.
    pub struct AstarState<'a, N: 'a, C: 'a> {
        /// The arena that all nodes are allocated from.
        pub arena: TypedArena<Node<'a, N, C>>,
        /// The priority queue that orders nodes in increasing
        /// cost + heuristic.
        pub queue: RefCell<PriorityQueue<Wnode<'a, N, C>>>,
        /// A hashmap of all of the nodes that we've visited recently.
        /// This should be a hashset, but rust won't let me grab keys
        /// from a hashset and mutate them.
        pub seen: RefCell<HashMap<&'a Node<'a, N, C>, &'a Node<'a, N, C>>>,
    }

    impl <'a, N, C> AstarState<'a, N, C>
    where N: Hash + PartialEq, C: PartialOrd {
        pub fn new() -> AstarState<'a, N, C> {
            AstarState {
                arena: TypedArena::new(),
                queue: RefCell::new(PriorityQueue::new()),
                seen: RefCell::new(HashMap::new())
            }
        }

        /// Creates a node from a state, a cost, and a cost + heuristic.
        /// This node gets added to the set of open nodes, and gets added to
        /// the priority queue.
        pub fn add(&'a self, n: N, cost: C, heur_cost: C) -> &'a Node<'a, N, C> {
            let node = Node {
                state: RefCell::new(Some(n)),
                parent: RefCell::new(None),
                cost: RefCell::new(cost),
                open: RefCell::new(true),
                cost_with_heuristic: RefCell::new(heur_cost)
            };

            let node_ref = self.arena.alloc(node);
            let wrapped = Wnode::new(node_ref);
            self.queue.borrow_mut().push(wrapped);
            self.seen.borrow_mut().insert(node_ref, node_ref);

            node_ref
        }

        /// Removes the node from the open set with the smallest
        /// cost + heuristic.
        /// Places the node in the closed set.
        pub fn pop(&'a self) -> Option<&'a Node<'a, N, C>> {
            let node = self.queue.borrow_mut().pop().map(|w| w.node);
            match node {
                Some(node) => {
                    *node.open.borrow_mut() = false;
                }
                None => {}
            }
            node
        }

        /// Returns true if the node with a given state is in the closed set.
        pub fn is_closed(&self, state: &N) -> bool {
            match self.seen.borrow().find_equiv(&DumbNode(state)) {
                Some(node) => *node.open.borrow(),
                None => false,
            }
        }

        /// Returns a node with a given state if that node exists and is open.
        pub fn find_open(&self, state: &N) -> Option<&'a Node<'a, N, C>> {
            match self.seen.borrow().find_equiv(&DumbNode(state)).map(|a| *a) {
                Some(n) if *n.open.borrow() => Some(n),
                _ => None
            }
        }
    }
}

/// A SearchProblem is a description of the problem that will be solved with A*.
/// Implementing this trait will describe the problem well enough that it can
/// be solved without any more information.
/// N is the type of one of the search-states and
/// C is the type of the cost to get from one state to another.
pub trait SearchProblem<N, C> {
    /// A state representing the start of the search.
    fn start(&self) -> N;
    /// Check to see if a state is the goal state.
    fn is_end(&self, &N) -> bool;
    /// A function that estimates the cost to get from
    /// a node to the end.
    /// heuristic(end_state) should always be 0.
    fn heuristic(&self, &N) -> C;
    /// A function returning the neighbors of a search state along
    /// with the cost to get to that state.
    fn neighbors(&self, at: &N) -> Vec<(N, C)>;
}

/// Perform an A* search on the provided search-problem.
pub fn astar<N, C, S: SearchProblem<N, C>>(s: S) -> Option<Vec<N>>
where N: Hash + PartialEq , C: PartialOrd + Zero + Clone {
    // Start out with a search-state that contains the beginning
    // node with cost zero.  Heuristic cost is also zero, but  this
    // shouldn't matter as it will be removed from the priority queue instantly.
    let state: state::AstarState<N, C> = state::AstarState::new();
    state.add(s.start(), Zero::zero(), Zero::zero());
    let mut end;

    loop {
        // Find the node with the smallest heuristic distance.
        let current = match state.pop() {
            // If we find the end node, start reconstructing the path.
            Some(ref node) if s.is_end(node.state.borrow().as_ref().unwrap()) => {
                end = Some(*node);
                break;
            }
            Some(node) => node,
            // If there are no more nodes in the queue, we have failed to
            // find a path.
            None => {
                return None;
            }
        };

        // Go through each neighbor to the current node and
        // either add it to the problem state, or update it if
        // necessary.
        for (neighbor_state, cost) in s.neighbors(
        current.state.borrow().as_ref().unwrap()).into_iter() {
            if state.is_closed(&neighbor_state) {
                continue;
            }

            let tentative_g_score = *current.cost.borrow() + cost;

            match state.find_open(&neighbor_state) {
                Some(n) if *n.cost.borrow() < tentative_g_score.clone() => {
                    *n.cost.borrow_mut() = tentative_g_score.clone();
                    let heur = s.heuristic(&neighbor_state);
                    *n.cost_with_heuristic.borrow_mut() = tentative_g_score + heur;
                    *n.parent.borrow_mut() = Some(current)
                }
                Some(_) => {}
                None => {
                    let heur = s.heuristic(&neighbor_state);
                    let n = state.add(neighbor_state,
                                  tentative_g_score.clone(),
                                  tentative_g_score + heur);
                    *n.parent.borrow_mut() = Some(current)
                }
            };
        }
    }

    // If we've reached this point, then a valid path exists from the start
    // to the end.  Construct this path by traversing backwards from the end
    // back to the start via the parent property.
    let mut cur = end;
    let mut path = vec![];
    loop {
        match cur {
            Some(n) => {
                path.push(n.state.borrow_mut().take().unwrap());
                cur = *n.parent.borrow();
            }
            None => {
                return Some(path);
            }
        }
    }
}
