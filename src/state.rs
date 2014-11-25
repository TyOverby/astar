use arena::TypedArena;
use super::node::{Node, DumbNode};
use super::wrap::WrappedNode as Wnode;
use std::intrinsics::transmute;
use std::collections::{BinaryHeap, HashMap};
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
    pub queue: RefCell<BinaryHeap<Wnode<'a, N, C>>>,
    /// A hashmap of all of the nodes that we've visited recently.
    /// This should be a hashset, but rust won't let me grab keys
    /// from a hashset and mutate them.
    pub seen: RefCell<HashMap<DumbNode<'a, N>, &'a Node<'a, N, C>>>,
}

impl <'a, N, C> AstarState<'a, N, C>
where N: Hash + PartialEq, C: PartialOrd {
    pub fn new() -> AstarState<'a, N, C> {
        AstarState {
            arena: TypedArena::new(),
            queue: RefCell::new(BinaryHeap::new()),
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
        let b = node_ref.state.borrow();
        self.seen.borrow_mut().insert(DumbNode(unsafe { transmute(b.as_ref().unwrap()) }), node_ref);

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
    pub fn is_closed(&'a self, state: &N) -> bool {
        let state = unsafe { transmute(state) };
        match self.seen.borrow().get(&DumbNode(state)) {
            Some(node) => *node.open.borrow(),
            None => false,
        }
    }

    /// Returns a node with a given state if that node exists and is open.
    pub fn find_open(&'a self, state: &N) -> Option<&'a Node<'a, N, C>> {
        let state = unsafe { transmute(state) };
        match self.seen.borrow().get(&DumbNode(state)).map(|a| *a) {
            Some(n) if *n.open.borrow() => Some(n),
            _ => None
        }
    }
}
