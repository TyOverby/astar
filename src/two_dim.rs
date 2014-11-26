use std::cell::RefCell;
use std::collections::RingBuf;
use std::vec::MoveItems;
use num::{Zero, One};

use super::{SearchProblem, astar};

struct TwoDimSearchProblemWrapper<'a, Rsp: 'a> {
    start: RefCell<Option<(i32, i32)>>,
    end: (i32, i32),
    rsp: &'a Rsp
}

pub trait TwoDimSearchProblem<C: One> {
    fn get(&self, x: i32, y: i32) -> Option<C>;
    fn diag(&self) -> bool { false }
    fn tween(&self) -> bool { false }
}

impl <'a, C, Rsp> SearchProblem<(i32, i32), C, MoveItems<((i32,i32), C)>> for TwoDimSearchProblemWrapper<'a, Rsp>
where C: Zero, Rsp: TwoDimSearchProblem<C>
{
    fn start(&self) -> (i32, i32) {
        self.start.borrow_mut().take().unwrap()
    }
    fn is_end(&self, node: &(i32, i32)) -> bool {
        (&self.end) == node
    }
    fn heuristic(&self, node: &(i32, i32)) -> C {
        Zero::zero()
    }
    fn neighbors(&self, node: &(i32, i32)) -> MoveItems<((i32, i32), C)> {
        (vec![]).into_iter()
    }
}



pub fn astar_t<C, S>(s: &S, start: (i32, i32), end: (i32, i32)) -> Option<RingBuf<(i32, i32)>> where
C: PartialOrd + Zero + Clone + One,
S: TwoDimSearchProblem<C>
{
    let rspw = TwoDimSearchProblemWrapper {
        start: RefCell::new(Some(start)),
        end: end,
        rsp: s
    };

    astar(rspw)
}


