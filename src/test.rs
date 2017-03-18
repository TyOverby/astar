use super::{astar, SearchProblem, TwoDSearchProblem, ReusableSearchProblem};
use std::collections::VecDeque;
use std::vec::IntoIter;

struct GridState {
    start: (i32, i32),
    end: (i32, i32)
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

    fn start(&self) -> (i32, i32) { self.start }
    fn is_end(&self, other: &(i32, i32)) -> bool { other == &self.end }
    fn heuristic(&self, &(p_x, p_y): &(i32, i32)) -> i32 {
        let (s_x, s_y) = self.end;
        abs(s_x - p_x) + abs(s_y - p_y)
    }
    fn neighbors(&mut self, &(x, y): &(i32, i32)) -> IntoIter<((i32, i32), i32)> {
        let mut vec = vec![];
        for i in -1 .. 1 + 1 {
            for k in -1 .. 1 + 1 {
                if !(i == 0 && k == 0) {
                    vec.push(((x + i, y + k), 1));
                }
            }
        }
        vec.into_iter()
    }
}

fn path(start: (i32, i32), end: (i32, i32)) -> Option<VecDeque<(i32, i32)>> {
    let mut gs = GridState{ start: start, end: end };
    astar(&mut gs)
}

#[test]
fn test_iter() {
    let mut gs = GridState{ start: (0,0), end: (0,0) };
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
    assert_eq!(p, vec![(0, 0)].into_iter().collect::<VecDeque<_>>());
}

#[test]
fn test_next() {
    let p = path((0,0), (0,1)).unwrap();
    assert_eq!(p, vec![(0,0), (0,1)].into_iter().collect::<VecDeque<_>>());
}

#[test]
fn test_few() {
    let p = path((0,0), (0,4)).unwrap();
    assert_eq!(p, vec![(0,0), (0,1) ,(0,2), (0,3), (0,4)].into_iter().collect::<VecDeque<_>>());
}

struct Maze {
    xmax: i32,
    ymax: i32
}

impl TwoDSearchProblem for Maze {
    fn get(&mut self, x: i32, y: i32) -> Option<u32> {
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
    let mut maze = Maze{ xmax: 7, ymax: 5 };
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
    let p = astar(&mut maze.search((0,0), (0,4))).unwrap();
    assert_eq!(p, vec![
        (0, 0),
        (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1),
        (7, 2),
        (7, 3), (6, 3), (5, 3), (4, 3), (3, 3), (2, 3), (1, 3), (0, 3),
        (0, 4)].into_iter().collect());
}

#[test]
fn test_maze_reusable() {
    let mut maze = Maze{ xmax: 7, ymax: 5 };
    let p = astar(&mut maze.search((0,0), (0,4))).unwrap();
    let p2 = astar(&mut maze.search((0,0), (0,4))).unwrap();
    assert_eq!(p, p2);
}

#[test]
fn test_maze_reverse() {
    let mut maze = Maze{ xmax: 7, ymax: 5 };
    let p = astar(&mut maze.search((0,0), (0,4))).unwrap();
    let p2 = astar(&mut maze.search((0,4), (0,0))).unwrap();
    assert_eq!(p, p2.into_iter().rev().collect());
}
