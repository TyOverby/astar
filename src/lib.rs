extern crate arena;
extern crate num;

use std::hash::Hash;
use std::cell::RefCell;
use std::collections::RingBuf;
use num::Zero;

#[cfg(test)]
mod test;
mod node;
mod wrap;
mod state;

struct ReusableSearchProblemWrapper<'a, N, Rsp: 'a> {
    start: RefCell<Option<N>>,
    end: N,
    rsp: &'a Rsp
}


impl <'a, N, C, I: Iterator<(N, C)>, Rsp> SearchProblem<N, C, I> for ReusableSearchProblemWrapper<'a, N, Rsp>
where N: PartialEq, Rsp: ReusableSearchProblem<N, C, I>
{
    fn start(&self) -> N { return self.start.borrow_mut().take().unwrap() }
    fn is_end(&self, node: &N) -> bool { (&self.end) == node }
    fn heuristic(&self, node: &N) -> C { self.rsp.heuristic(node) }
    fn neighbors(&self, node: &N) -> I { self.rsp.neighbors(node) }
}

/// A SearchProblem is a description of the problem that will be solved with A*.
/// Implementing this trait will describe the problem well enough that it can
/// be solved without any more information.
/// N is the type of one of the search-states and
/// C is the type of the cost to get from one state to another.
pub trait SearchProblem<N, C, I: Iterator<(N, C)>> {
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
    fn neighbors(&self, at: &N) -> I;
    /// This method is used if an estimated length of the path
    /// is available.
    fn estimate_length(&self) -> Option<uint> { None }
}

/// ReusableSearchProblem is like a regular SearchProblem but without
/// the `start()` and `is_end()` checks.  Instead, the start and end
/// will be provided when `astar_r()` is called.
pub trait ReusableSearchProblem<N, C, I: Iterator<(N, C)>> {
    /// A function that estimates the cost to get from
    /// a node to the end.
    /// heuristic(end_state) should always be 0.
    fn heuristic(&self, &N) -> C;
    /// A function returning the neighbors of a search state along
    /// with the cost to get to that state.
    fn neighbors(&self, at: &N) -> I;
    /// This method is used if an estimated length of the path
    /// is available.
    fn estimate_length(&self) -> Option<uint> { None }
}

/// Perform an A* search on the provided search-problem.
pub fn astar<N, C, I, S: SearchProblem<N, C, I>>(s: S) -> Option<RingBuf<N>>
where N: Hash + PartialEq,
      C: PartialOrd + Zero + Clone,
      I: Iterator<(N, C)> {
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
        current.state.borrow().as_ref().unwrap()) {
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
    let mut path = RingBuf::with_capacity(s.estimate_length().unwrap_or(16));
    loop {
        match cur {
            Some(n) => {
                path.push_front(n.state.borrow_mut().take().unwrap());
                cur = *n.parent.borrow();
            }
            None => {
                return Some(path);
            }
        }
    }
}

pub fn astar_r<N, C, I, S: ReusableSearchProblem<N, C, I>>(s: &S, start: N, end: N) -> Option<RingBuf<N>>
where N: Hash + PartialEq,
      C: PartialOrd + Zero + Clone,
      I: Iterator<(N, C)>
{
    let rspw = ReusableSearchProblemWrapper {
        start: RefCell::new(Some(start)),
        end: end,
        rsp: s
    };

    astar(rspw)
}


