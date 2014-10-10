use std::collections::PriorityQueue;
use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::num::Num;
use std::num::Zero;

use node_wrapper::NodeWrapper as Nwrap;

// Required for making a min-heap
mod reverse_ord;
mod node_wrapper;

pub trait SearchState<N, C> {
    fn start(&self) -> N;
    fn end(&self) -> N;
    fn heuristic(&self, from: &N) -> C;
    fn neighbors(&self, at: &N) -> Vec<(N, C)>;
}

pub trait Node: Hash + Clone + Eq {}
pub trait Cost: Num + Zero + Ord + Clone {}

pub fn astar<N, C, S: SearchState<N, C>>(s: S) -> Option<Vec<N>>
where N: Hash + Clone + Eq, C: Num + Zero + Ord + Clone {
    let start = s.start();
    let end = s.end();

    let mut closed_set: HashSet<N> = HashSet::new();
    let mut open_set = HashSet::new();
    let mut open_queue = PriorityQueue::new();
    let mut g_score = HashMap::new();
    let mut f_score = HashMap::new();
    let mut came_from = HashMap::new();

    open_set.insert(start.clone());
    let zero: C = Zero::zero();
    open_queue.push(Nwrap::new(start.clone(), zero.clone()));
    g_score.insert(start.clone(), zero);
    f_score.insert(start.clone(), s.heuristic(&start));

    loop {
        let current = match open_queue.pop() {
            Some(Nwrap{n: ref x, ..}) if x == &end => { break; }
            Some(Nwrap{n: x, ..}) => x,
            None => { return None; }
        };
        open_set.remove(&current);
        closed_set.insert(current.clone());

        for (neighbor, dist) in s.neighbors(&current).into_iter() {
            if closed_set.contains(&neighbor) {
                continue;
            }
            let tentative_g_score = g_score[current.clone()] + dist;
            if open_set.contains(&neighbor) ||
               !g_score.contains_key(&neighbor) ||
                g_score[neighbor.clone()] > tentative_g_score.clone() {

                came_from.insert(neighbor.clone(), current.clone());
                g_score.insert(neighbor.clone(), tentative_g_score.clone());
                let total = tentative_g_score + s.heuristic(&neighbor);
                f_score.insert(neighbor.clone(), total.clone());
                if open_set.contains(&neighbor) {
                    open_set.insert(neighbor.clone());
                    open_queue.push(Nwrap::new(neighbor, total));
                }
            }
        }
    }

    let mut path = vec![];
    let mut cur = Some(end);

    loop {
        match cur.take() {
            Some(ref node) if *node == start => {
                path.push(start);
                path.reverse();
                return Some(path);
            }
            Some(node) => {
                cur = came_from.pop(&node);
                path.push(node);
            }
            None => return None
        }

    }
}
