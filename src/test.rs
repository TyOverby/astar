use super::{astar, SearchProblem};
use num_traits::Zero;
use std::collections::VecDeque;
use std::hash::Hash;
use std::vec::IntoIter;

pub trait ReusableSearchProblem {
    type Node: Hash + PartialEq + Eq + Clone;
    type Cost: PartialOrd + Zero + Clone;
    type Iter: Iterator<Item = (Self::Node, Self::Cost)>;

    fn heuristic(&self, &Self::Node, &Self::Node) -> Self::Cost;
    fn neighbors(&self, &Self::Node) -> Self::Iter;
    fn estimate_length(&self, _a: &Self::Node, _b: &Self::Node) -> Option<u32> {
        None
    }

    fn search(&mut self, end: Self::Node) -> ReuseSearchInstance<Self, Self::Node> {
        ReuseSearchInstance {
            rsp: self,
            end: end,
            estimation: None,
        }
    }
}

pub trait TwoDSearchProblem {
    fn get(&self, x: i32, y: i32) -> Option<u32>;
    fn diag(&self) -> bool {
        false
    }
    fn cut_corners(&self) -> bool {
        false
    }
}

pub struct ReuseSearchInstance<'a, RSP: 'a + ?Sized, S> {
    rsp: &'a RSP,
    end: S,
    estimation: Option<u32>,
}

impl<T: TwoDSearchProblem> ReusableSearchProblem for T {
    type Node = (i32, i32);
    type Cost = u32;
    type Iter = IntoIter<((i32, i32), u32)>;

    fn heuristic(&self, a: &Self::Node, b: &Self::Node) -> Self::Cost {
        fn abs(x: i32) -> i32 {
            if x < 0 {
                -x
            } else {
                x
            }
        }

        let &(bx, by) = a;
        let &(ex, ey) = b;
        let (dx, dy) = (ex - bx, ey - by);

        // Chebyshev Distance
        // return (max(abs(dx), abs(dy)) * 2) as u32;
        // Manhattan Distance
        return ((abs(dx) + abs(dy)) as u32) * 2;
    }

    fn neighbors(&self, node: &Self::Node) -> Self::Iter {
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

    fn estimate_length(&self, _a: &Self::Node, _b: &Self::Node) -> Option<u32> {
        None
    }
}

impl<'a, RSP: ReusableSearchProblem> SearchProblem for ReuseSearchInstance<'a, RSP, RSP::Node> {
    type Node = RSP::Node;
    type Cost = RSP::Cost;
    type Iter = RSP::Iter;

    fn is_end(&self, other: &Self::Node) -> bool {
        &self.end == other
    }

    fn heuristic(&self, a: &Self::Node) -> Self::Cost {
        self.rsp.heuristic(a, &self.end)
    }

    fn neighbors(&self, node: &Self::Node, _: &Self::Cost) -> Self::Iter {
        self.rsp.neighbors(node)
    }

    fn estimate_length(&self) -> Option<u32> {
        self.estimation
    }
}
struct GridState {
    start: (i32, i32),
    end: (i32, i32),
}

fn abs(x: i32) -> i32 {
    if x < 0 {
        -x
    } else {
        x
    }
}

impl SearchProblem for GridState {
    type Node = (i32, i32);
    type Cost = i32;
    type Iter = IntoIter<((i32, i32), i32)>;

    fn is_end(&self, other: &(i32, i32)) -> bool {
        other == &self.end
    }
    fn heuristic(&self, &(p_x, p_y): &(i32, i32)) -> i32 {
        let (s_x, s_y) = self.end;
        abs(s_x - p_x) + abs(s_y - p_y)
    }
    fn neighbors(&self, &(x, y): &(i32, i32), _: &i32) -> IntoIter<((i32, i32), i32)> {
        let mut vec = vec![];
        for i in -1..1 + 1 {
            for k in -1..1 + 1 {
                if !(i == 0 && k == 0) {
                    vec.push(((x + i, y + k), 1));
                }
            }
        }
        vec.into_iter()
    }
}

fn path(start: (i32, i32), end: (i32, i32)) -> Option<(VecDeque<(i32, i32)>, i32)> {
    let gs = GridState {
        start: start,
        end: end,
    };
    if let Some((p, c)) = astar(&gs, gs.start) {
        Some((p.into_iter().map(|(a, _)| a).collect(), c))
    } else {
        None
    }
}

#[test]
fn test_iter() {
    let gs = GridState {
        start: (0, 0),
        end: (0, 0),
    };
    assert!(
        gs.neighbors(&(0, 0), &0).collect::<Vec<_>>()
            == vec![
                ((-1, -1), 1),
                ((-1, 0), 1),
                ((-1, 1), 1),
                ((0, -1), 1),
                ((0, 1), 1),
                ((1, -1), 1),
                ((1, 0), 1),
                ((1, 1), 1),
            ]
    )
}

#[test]
fn test_start_end() {
    let p = path((0, 0), (0, 0)).unwrap();
    assert_eq!(p.0, vec![(0, 0)].into_iter().collect::<VecDeque<_>>());
}

#[test]
fn test_next() {
    let p = path((0, 0), (0, 1)).unwrap();
    assert_eq!(
        p.0,
        vec![(0, 0), (0, 1)].into_iter().collect::<VecDeque<_>>()
    );
}

#[test]
fn test_few() {
    let p = path((0, 0), (0, 4)).unwrap();
    assert_eq!(
        p.0,
        vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)]
            .into_iter()
            .collect::<VecDeque<_>>()
    );
}

struct Maze {
    xmax: i32,
    ymax: i32,
}

impl TwoDSearchProblem for Maze {
    fn get(&self, x: i32, y: i32) -> Option<u32> {
        /* Imagine a simple maze, that looks something like this:
        .
        ........
               .
        ........
        .
        ........
        where . is passable, and everywhere else is impassible.
        */
        if x < 0 || x > self.xmax || y < 0 || y > self.ymax {
            None
        } else if y % 4 == 0 && x > 0 {
            None
        } else if (y + 2) % 4 == 0 && x < self.xmax {
            None
        } else {
            Some(0)
        }
    }
}

#[test]
fn test_maze() {
    let mut maze = Maze { xmax: 7, ymax: 5 };
    /* If this test fails, try printing out the maze using this code:
    println!("");
    for y in 0 .. maze.ymax+1 {
        for x in 0 .. maze.xmax+1 {
            match maze.get(x, y) {
                Some(_) => print!("."),
                None => print!(" "),
            }
        }
        println!("");
    }
    */
    let p = astar(&mut maze.search((0, 4)), (0, 0)).unwrap();
    assert_eq!(
        p.0.into_iter().map(|(a, _)| a).collect::<VecDeque<_>>(),
        vec![
            (0, 0),
            (0, 1),
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 1),
            (5, 1),
            (6, 1),
            (7, 1),
            (7, 2),
            (7, 3),
            (6, 3),
            (5, 3),
            (4, 3),
            (3, 3),
            (2, 3),
            (1, 3),
            (0, 3),
            (0, 4),
        ].into_iter()
            .collect::<VecDeque<_>>()
    );
}

#[test]
fn test_maze_reusable() {
    let mut maze = Maze { xmax: 7, ymax: 5 };
    let p = astar(&mut maze.search((0, 4)), (0, 0)).unwrap();
    let p2 = astar(&mut maze.search((0, 4)), (0, 0)).unwrap();
    assert_eq!(p, p2);
}

#[test]
fn test_maze_reverse() {
    let mut maze = Maze { xmax: 7, ymax: 5 };
    let p = astar(&mut maze.search((0, 4)), (0, 0)).unwrap();
    let p2 = astar(&mut maze.search((0, 0)), (0, 4)).unwrap();
    assert_eq!(
        p.0.into_iter().map(|(a, _)| a).collect::<Vec<_>>(),
        p2.0.into_iter().rev().map(|(a, _)| a).collect::<Vec<_>>()
    );
}
