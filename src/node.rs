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
