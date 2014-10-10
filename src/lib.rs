use std::collections::PriorityQueue;
use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::num::Num;
use std::num::Zero;

use reverse_ord::ReverseOrd as Rord;

// Required for making a min-heap
mod reverse_ord;

trait SearchState<N, C> {
    fn start(&self) -> N;
    fn end(&self) -> N;
    fn heuristic(&self, from: &N) -> C;
    fn neighbors(&self, at: &N) -> Vec<(N, C)>;
}

trait Node: Hash + Clone {}
trait Cost: Num + Zero + PartialOrd + Clone {}

pub fn astar<N: Ord + Hash + Clone, F: Num + Zero + PartialOrd + Clone>(
        start: N,
        end: N,
        heuristic: |&N, &N| -> F,
        neighbors: |&N| -> Vec<(N, F)>
        ) -> Option<Vec<N>> {
    let mut closed_set: HashSet<N> = HashSet::new();
    let mut open_set = HashSet::new();
    let mut open_queue = PriorityQueue::new();
    let mut g_score = HashMap::new();
    let mut f_score = HashMap::new();
    let mut came_from = HashMap::new();

    open_set.insert(start.clone());
    open_queue.push(Rord::new(start.clone()));

    let zero: F = Zero::zero();
    g_score.insert(start.clone(), zero);
    f_score.insert(start.clone(), heuristic(&start, &end));

    loop {
        let current = match open_queue.pop() {
            Some(Rord{e: ref x}) if x == &end => { break; }
            Some(Rord{e: x}) => x,
            None => { return None; }
        };
        open_set.remove(&current);
        closed_set.insert(current.clone());

        for (neighbor, dist) in neighbors(&current).into_iter() {
            if closed_set.contains(&neighbor) {
                continue;
            }
            let tentative_g_score = g_score[current.clone()] + dist;
            if open_set.contains(&neighbor) ||
               !g_score.contains_key(&neighbor) ||
                g_score[neighbor.clone()] > tentative_g_score.clone() {

                came_from.insert(neighbor.clone(), current.clone());
                g_score.insert(neighbor.clone(), tentative_g_score.clone());
                f_score.insert(neighbor.clone(), tentative_g_score +
                               heuristic(&neighbor, &end));
                if open_set.contains(&neighbor) {
                    open_set.insert(neighbor.clone());
                    open_queue.push(Rord::new(neighbor))
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
