#![allow(mutable_transmutes)]
extern crate num;
extern crate typed_arena;

use typed_arena::Arena as TypedArena;
use std::vec::IntoIter;
use num::Zero;
use std::hash::Hash;
use std::collections::BinaryHeap;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::cmp::Ordering;
use std::cell::{Cell, RefCell};
use std::mem;

#[cfg(test)]
mod test;

pub trait SearchProblem {
    type Node: Hash + PartialEq + Eq;
    type Cost: PartialOrd + Zero + Clone;
    type Iter: Iterator<Item=(Self::Node, Self::Cost)>;

    fn start(&self) -> Self::Node;
    fn is_end(&self, &Self::Node) -> bool;
    fn heuristic(&self, &Self::Node) -> Self::Cost;
    fn neighbors(&mut self, &Self::Node) -> Self::Iter;

    fn estimate_length(&self) -> Option<u32> { None }
}

pub trait ReusableSearchProblem {
    type Node: Hash + PartialEq + Eq + Clone;
    type Cost: PartialOrd + Zero + Clone;
    type Iter: Iterator<Item=(Self::Node, Self::Cost)>;

    fn heuristic(&self, &Self::Node, &Self::Node) -> Self::Cost;
    fn neighbors(&mut self, &Self::Node) -> Self::Iter;
    fn estimate_length(&self, _a: &Self::Node, _b: &Self::Node) -> Option<u32> { None }

    fn search(&mut self, start: Self::Node, end: Self::Node) -> ReuseSearchInstance<Self, Self::Node> {
        let est = self.estimate_length(&start, &end);
        ReuseSearchInstance {
            rsp: self,
            start: start,
            end: end,
            estimation: est
        }
    }
}

pub trait TwoDSearchProblem {
    fn get(&mut self, x: i32, y: i32) -> Option<u32>;
    fn diag(&self) -> bool { false }
    fn cut_corners(&self) ->  bool { false }
}

pub struct ReuseSearchInstance<'a, RSP: 'a + ?Sized, S> {
    rsp: &'a mut RSP,
    start: S,
    end: S,
    estimation: Option<u32>
}

impl <T: TwoDSearchProblem> ReusableSearchProblem for T {
    type Node = (i32, i32);
    type Cost = u32;
    type Iter = IntoIter<((i32, i32), u32)>;

    fn heuristic(&self, a: &Self::Node, b: &Self::Node) -> Self::Cost {
        fn abs(x: i32) -> i32 {
            if x < 0 { -x  } else { x }
        }

        let &(bx, by) = a;
        let &(ex, ey) = b;
        let (dx, dy) = (ex - bx, ey - by);

        // Chebyshev Distance
        // return (max(abs(dx), abs(dy)) * 2) as u32;
        // Manhattan Distance
        return ((abs(dx) + abs(dy)) as u32) * 2;
    }

    fn neighbors(&mut self, node: &Self::Node) -> Self::Iter {
                let mut v = vec![];
        let (x, y) = *node;
        let ap = (x + 0, y + 1);
        let bp = (x - 1, y + 0);
        let cp = (x + 1, y + 0);
        let dp = (x + 0, y - 1);

        let a = self.get(ap.0, ap.1);
        let b = self.get(bp.0, bp.1);
        let c = self.get(cp.0, cp.1);
        let d = self.get(dp.0, dp.1);

        if let Some(p) = a {
            v.push((ap, p + 2));
        }
        if let Some(p) = b {
            v.push((bp, p + 2));
        }
        if let Some(p) = c {
            v.push((cp, p + 2));
        }
        if let Some(p) = d {
            v.push((dp, p + 2));
        }

        if self.diag() {
            let xp = (x - 1, y + 1);
            let yp = (x + 1, y + 1);
            let zp = (x - 1, y - 1);
            let wp = (x + 1, y - 1);

            if !self.cut_corners() {
                if a.is_some() && b.is_some() {
                    if let Some(p) = self.get(xp.0, xp.1) {
                        v.push((xp, p + 3));
                    }
                }
                if a.is_some() && c.is_some() {
                    if let Some(p) = self.get(yp.0, yp.1) {
                        v.push((yp, p + 3));
                    }
                }
                if c.is_some() && d.is_some() {
                    if let Some(p) = self.get(wp.0, wp.1) {
                        v.push((wp, p + 3));
                    }
                }
                if b.is_some() && d.is_some() {
                    if let Some(p) = self.get(zp.0, zp.1) {
                        v.push((zp, p + 3))
                    }
                }

            } else {
                if let Some(p) = self.get(xp.0, xp.1) {
                    v.push((xp, p + 3));
                }
                if let Some(p) = self.get(yp.0, yp.1) {
                    v.push((yp, p + 3));
                }
                if let Some(p) = self.get(wp.0, wp.1) {
                    v.push((wp, p + 3));
                }
                if let Some(p) = self.get(zp.0, zp.1) {
                    v.push((zp, p + 3))
                }
            }
        }

        v.into_iter()
    }

    fn estimate_length(&self, _a: &Self::Node, _b: &Self::Node) -> Option<u32> { None }
}

impl <'a, RSP: ReusableSearchProblem> SearchProblem for ReuseSearchInstance<'a, RSP, RSP::Node> {
    type Node = RSP::Node;
    type Cost = RSP::Cost;
    type Iter = RSP::Iter;

    fn start(&self) -> Self::Node {
         self.start.clone()
    }

    fn is_end(&self, other: &Self::Node) -> bool {
        &self.end == other
    }

    fn heuristic(&self, a: &Self::Node) -> Self::Cost {
        self.rsp.heuristic(a, &self.end)
    }

    fn neighbors(&mut self, node: &Self::Node) -> Self::Iter {
        self.rsp.neighbors(node)
    }

    fn estimate_length(&self) -> Option<u32> { self.estimation }
}

#[derive(Debug)]
struct SearchNode<'a: 'b, 'b, S: 'a , C: Clone + 'a> {
    pub state: &'a S,
    pub parent: RefCell<Option<&'b SearchNode<'a, 'b, S, C>>>,

    pub g: RefCell<C>,
    pub f: RefCell<C>,
    pub h: RefCell<C>,

    pub opened: Cell<bool>,
    pub closed: Cell<bool>,
}

impl <'a, 'b, S, C: Zero + Clone> SearchNode<'a, 'b, S, C> {
    fn new_initial(state: &'a S) -> SearchNode<S, C> {
        SearchNode {
            state: state,
            parent: RefCell::new(None),
            g: RefCell::new(Zero::zero()),
            f: RefCell::new(Zero::zero()),
            h: RefCell::new(Zero::zero()),
            opened: Cell::new(true),
            closed: Cell::new(false)
        }
    }

    fn new(state: &'a S) -> SearchNode<S, C> {
        SearchNode {
            state: state,
            parent: RefCell::new(None),
            g: RefCell::new(Zero::zero()),
            f: RefCell::new(Zero::zero()),
            h: RefCell::new(Zero::zero()),
            opened: Cell::new(false),
            closed: Cell::new(false)
        }
    }

    fn g(&self) -> C {
        self.g.borrow().clone()
    }

    fn h(&self) -> C {
        self.h.borrow().clone()
    }

    fn set_g(&self, g: C) {
        *self.g.borrow_mut() = g;
    }

    fn set_f(&self, f: C) {
        *self.f.borrow_mut() = f;
    }

    fn set_h(&self, h: C) {
        *self.h.borrow_mut() = h;
    }

    fn set_parent(&self, p: &'b SearchNode<'a, 'b, S, C>) {
        *self.parent.borrow_mut() = Some(p);
    }
}

impl <'a, 'b, S: PartialEq, C: Clone> PartialEq for SearchNode<'a, 'b, S, C> {
    fn eq(&self, other: &SearchNode<S, C>) -> bool {
        self.state.eq(&other.state)
    }
}

impl <'a, 'b, S: PartialEq, C: Clone> Eq for SearchNode<'a, 'b, S, C> {}

impl<'a, 'b, S: PartialEq, C: PartialOrd + Clone> PartialOrd for SearchNode<'a, 'b, S, C> {
    fn partial_cmp(&self, other: &SearchNode<S, C>) -> Option<Ordering> {
        other.f.borrow().partial_cmp(&self.f.borrow())
    }
}

impl<'a, 'b, S: PartialEq, C: PartialOrd + Clone> Ord for SearchNode<'a, 'b, S, C> {
    fn cmp(&self, other: &SearchNode<'a, 'b, S, C>) -> Ordering {
        match self.partial_cmp(other) {
            Some(x) => x,
            None => Ordering::Equal
        }
    }
}

// We aren't correctly closing items and they are getting popped multiple times.
pub fn astar<S: SearchProblem>(s: &mut S) -> Option<VecDeque<S::Node>> where S::Node: ::std::clone::Clone + ::std::fmt::Debug, S::Cost: ::std::fmt::Debug {
    let state_arena: TypedArena<S::Node>  = TypedArena::new();
    let node_arena: TypedArena<SearchNode<S::Node, S::Cost>> = TypedArena::new();

    let mut state_to_node: HashMap<&S::Node, &SearchNode<S::Node, S::Cost>> = HashMap::new();
    let mut heap: BinaryHeap<&SearchNode<S::Node, S::Cost>> = BinaryHeap::new();

    let start_state: &S::Node = state_arena.alloc(s.start());

    let start_node: SearchNode<S::Node, S::Cost> = SearchNode::new_initial(start_state);
    let start_node: &SearchNode<S::Node, S::Cost> = node_arena.alloc(start_node);
    state_to_node.insert(start_state, start_node);

    heap.push(start_node);

    let mut found = None;

    while let Some(node) = heap.pop() {
        let node_state = node.state;

        node.closed.set(true);
        node.opened.set(false);

        if s.is_end(node_state) {
            found = Some(node);
            break;
        }

        for (neighbor, cost) in s.neighbors(node_state) {
            let neighbor_state:&_ = state_arena.alloc(neighbor);
            let neighbor_node =
                state_to_node.entry(neighbor_state)
                             .or_insert_with(|| node_arena.alloc(SearchNode::new(neighbor_state)));

            if neighbor_node.closed.get() {
                continue;
            }

            let ng = node.g() + cost;
            if !neighbor_node.opened.get() || ng < neighbor_node.g() {
                let h = if neighbor_node.h() == Zero::zero() {
                    s.heuristic(neighbor_state)
                } else {
                    neighbor_node.h()
                };

                neighbor_node.set_g(ng.clone());
                neighbor_node.set_h(h.clone());
                neighbor_node.set_f(ng + h);
                neighbor_node.set_parent(node);

                if !neighbor_node.opened.get() {
                    neighbor_node.opened.set(true);
                    heap.push(neighbor_node);
                } else {
                    // We reset the value that did sorting.  This forces a
                    // recalculation.
                    heap = heap_from_vec(vec_from_heap(heap)) // BinaryHeap::from_vec(heap.into_vec());
                }
            }
        }
    }

    if found.is_some() {
        let mut prev = found;
        let mut deque = VecDeque::new();

        while let Some(node) = prev {
            deque.push_front((*node.state).clone());
            prev = node.parent.borrow_mut().take();
        }

        Some(deque)
    } else {
        None
    }
}

fn heap_from_vec<T: Ord>(v: Vec<T>) -> BinaryHeap<T> {
    let mut b_heap = BinaryHeap::with_capacity(v.len());
    b_heap.extend(v.into_iter());
    b_heap
}

fn vec_from_heap<T: Ord>(h: BinaryHeap<T>) -> Vec<T> {
    h.into_iter().collect()
}
