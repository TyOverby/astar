use super::{astar, SearchProblem};
use std::iter::range_inclusive;
use std::collections::RingBuf;
use std::vec::MoveItems;

struct GridState {
    start: (i32, i32),
    end: (i32, i32)
}

impl SearchProblem<(i32, i32), i32, MoveItems<((i32, i32), i32)>> for GridState {
    fn start(&self) -> (i32, i32) { self.start }
    fn is_end(&self, p: &(i32, i32)) -> bool { *p == self.end }
    fn heuristic(&self, &(p_x, p_y): &(i32, i32)) -> i32 {
        let (s_x, s_y) = self.end;
        (s_x - p_x).abs() + (s_y - p_y).abs()
    }
    fn neighbors(&self, &(x, y): &(i32, i32)) -> MoveItems<((i32, i32), i32)> {
        let mut vec = vec![];
        for i in range_inclusive(-1, 1) {
            for k in range_inclusive(-1, 1) {
                if !(i == 0 && k == 0) {
                    vec.push(((x + i, y + k), 1));
                }
            }
        }
        vec.into_iter()
    }
}

fn path(start: (i32, i32), end: (i32, i32)) -> Option<RingBuf<(i32, i32)>> {
    let gs = GridState{ start: start, end: end };
    astar(gs)
}

#[test]
fn test_iter() {
    let gs = GridState{ start: (0,0), end: (0,0) };
    assert!(
        gs.neighbors(&(0,0)).collect::<Vec<_>>() ==
        vec![
            ((-1, -1), 1),
            ((-1, 0), 1),
            ((-1, 1), 1),
            ((0, -1), 1),
            ((0, 1), 1),
            ((1, -1), 1),
            ((1, 0), 1),
            ((1, 1), 1)
        ])
}

#[test]
fn test_start_end() {
    let p = path((0,0), (0,0)).unwrap();
    assert!(p == vec![(0, 0)].into_iter().collect());
}

#[test]
fn test_next() {
    let p = path((0,0), (0,1)).unwrap();
    assert!(p == vec![(0,0), (0,1)].into_iter().collect());
}

#[test]
fn test_few() {
    let p = path((0,0), (0,4)).unwrap();
    assert!(p == vec![(0,0), (0,1) ,(0,2), (0,3), (0,4)].into_iter().collect());
}
