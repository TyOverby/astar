extern crate arena;
extern crate debug;

use std::hash::Hash;
use std::num::Zero;

#[cfg(test)]
mod test;

mod node {
    use std::cell::RefCell;
    use std::hash::Hash;
    use std::hash::sip::SipState;
    use std::cmp::Equiv;

    #[deriving(Hash)]
    pub struct DumbNode<'b, N: 'b>(pub &'b N);

    impl <'a, 'b, N: PartialEq, C> Equiv<&'a Node<'a, N, C>> for DumbNode<'b, N> {
        fn equiv(&self, other: &&Node<'a, N, C>) -> bool {
            let DumbNode(x) = *self;
            x.eq(other.state.borrow().deref().as_ref().unwrap())
        }
    }

    pub struct Node<'a, N: 'a, C: 'a> {
        pub state: RefCell<Option<N>>,
        pub parent: RefCell<Option<&'a Node<'a, N, C>>>,
        pub open: RefCell<bool>,
        pub cost: RefCell<C>,
        pub cost_with_heuristic: RefCell<C>
    }

    impl <'a, N, C> Hash for Node<'a, N, C>
    where N: Hash {
        fn hash(&self, hash_state: &mut SipState) {
            self.state.borrow().as_ref().unwrap().hash(hash_state);
        }
    }

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

    impl <'a, N, C> PartialEq for WrappedNode<'a, N, C>
    where C: PartialEq {
        fn eq(&self, other: &WrappedNode<'a, N, C>) -> bool {
            let this_cost = self.node.cost.borrow();
            let that_cost = other.node.cost.borrow();
            *this_cost == *that_cost
        }
    }

    impl <'a, N, C> Eq for WrappedNode<'a, N, C> where C: PartialEq { }

    impl <'a, N, C> PartialOrd for WrappedNode<'a, N, C>
    where C: PartialOrd {
        fn partial_cmp(&self, other: &WrappedNode<'a, N, C>) -> Option<Ordering> {
            let this_cost = self.node.cost_with_heuristic.borrow();
            let that_cost = other.node.cost_with_heuristic.borrow();
            // Match backwards for the priority queue.
            (*that_cost).partial_cmp(&*this_cost)
        }
    }

    impl <'a, N, C> Ord for WrappedNode<'a, N, C>
    where C: PartialOrd {
        fn cmp(&self, other: &WrappedNode<'a, N, C>) -> Ordering {
            let this_cost = self.node.cost_with_heuristic.borrow();
            let that_cost = other.node.cost_with_heuristic.borrow();
            // Match backwards for the priority queue.
            match (*that_cost).partial_cmp(&*this_cost) {
                None => Equal,
                Some(x) => x
            }
        }
    }
}

mod state {
    use arena::TypedArena;
    use super::node::Node;
    use super::node::DumbNode;
    use super::wrap::WrappedNode as Wnode;
    use std::collections::PriorityQueue;
    use std::cell::RefCell;
    use std::hash::Hash;
    use std::collections::HashMap;

    pub struct AstarState<'a, N: 'a, C: 'a> {
        pub arena: TypedArena<Node<'a, N, C>>,
        pub queue: RefCell<PriorityQueue<Wnode<'a, N, C>>>,
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

        pub fn is_closed(&self, state: &N) -> bool {
            match self.seen.borrow().find_equiv(&DumbNode(state)) {
                Some(node) => *node.open.borrow(),
                None => false,
            }
        }

        pub fn find_open(&self, state: &N) -> Option<&'a Node<'a, N, C>> {
            match self.seen.borrow().find_equiv(&DumbNode(state)).map(|a| *a) {
                Some(n) if *n.open.borrow() => Some(n),
                _ => None
            }
        }
    }
}

pub trait SearchState<N, C> {
    fn start(&self) -> N;
    fn is_end(&self, &N) -> bool;
    fn heuristic(&self, &N) -> C;
    fn neighbors(&self, at: &N) -> Vec<(N, C)>;
}

pub fn astar<N, C, S: SearchState<N, C>>(s: S) -> Option<Vec<N>>
where N: Hash + PartialEq , C: PartialOrd + Zero + Clone {
    let state: state::AstarState<N, C> = state::AstarState::new();
    let mut end;
    state.add(s.start(), Zero::zero(), Zero::zero());

    loop {
        let current = match state.pop() {
            Some(ref node) if s.is_end(node.state.borrow().as_ref().unwrap()) => {
                end = Some(*node);
                break;
            }
            Some(node) => node,
            None => {
                return None;
            }
        };

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
