use std::cell::RefCell;
use std::collections::RingBuf;
use std::vec::MoveItems;

use super::{SearchProblem, astar};

struct TwoDimSearchProblemWrapper<'a, Rsp: 'a> {
    start: RefCell<Option<(i32, i32)>>,
    end: (i32, i32),
    rsp: &'a Rsp
}

pub trait TwoDimSearchProblem {
    #[inline(always)]
    fn get(&self, x: i32, y: i32) -> Option<u32>;
    #[inline(always)]
    fn diag(&self) -> bool { false }
    #[inline(always)]
    fn tween(&self) -> bool { false }
}

impl <'a, Rsp: TwoDimSearchProblem> SearchProblem<(i32, i32), u32, MoveItems<((i32,i32), u32)>>
for TwoDimSearchProblemWrapper<'a, Rsp> {
    fn start(&self) -> (i32, i32) {
        self.start.borrow_mut().take().unwrap()
    }
    fn is_end(&self, node: &(i32, i32)) -> bool {
        (&self.end) == node
    }
    fn heuristic(&self, node: &(i32, i32)) -> u32 {
        fn abs(x: i32) -> i32 {
            if x < 0 { -x  } else { x }
        }
        let &(bx, by) = node;
        let (ex, ey) = self.end;
        let (dx, dy) = (ex - bx, ey - by);

        (abs(dx) + abs(dy)) as u32
    }

    fn estimate_length(&self) -> Option<uint> {
        Some(self.heuristic(&self.start()) as uint)
    }

    fn neighbors(&self, node: &(i32, i32)) -> MoveItems<((i32, i32), u32)> {
        let mut v = vec![];
        let (x, y) = *node;
        let ap = (x + 0, y + 1);
        let bp = (x - 1, y + 0);
        let cp = (x + 1, y + 0);
        let dp = (x + 0, y - 1);

        let a = self.rsp.get(ap.0, ap.1);
        let b = self.rsp.get(bp.0, bp.1);
        let c = self.rsp.get(cp.0, cp.1);
        let d = self.rsp.get(dp.0, dp.1);

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

        if self.rsp.diag() {
            let xp = (x - 1, y + 1);
            let yp = (x + 1, y + 1);
            let zp = (x - 1, y - 1);
            let wp = (x + 1, y - 1);

            if !self.rsp.tween() {
                if a.is_some() && b.is_some() {
                    if let Some(p) = self.rsp.get(xp.0, xp.1) {
                        v.push((xp, p + 3));
                    }
                }
                if a.is_some() && c.is_some() {
                    if let Some(p) = self.rsp.get(yp.0, yp.1) {
                        v.push((yp, p + 3));
                    }
                }
                if c.is_some() && d.is_some() {
                    if let Some(p) = self.rsp.get(wp.0, wp.1) {
                        v.push((wp, p + 3));
                    }
                }
                if b.is_some() && d.is_some() {
                    if let Some(p) = self.rsp.get(zp.0, zp.1) {
                        v.push((zp, p + 3))
                    }
                }

            } else {
                if let Some(p) = self.rsp.get(xp.0, xp.1) {
                    v.push((xp, p + 3));
                }
                if let Some(p) = self.rsp.get(yp.0, yp.1) {
                    v.push((yp, p + 3));
                }
                if let Some(p) = self.rsp.get(wp.0, wp.1) {
                    v.push((wp, p + 3));
                }
                if let Some(p) = self.rsp.get(zp.0, zp.1) {
                    v.push((zp, p + 3))
                }
            }
        }

        v.into_iter()
    }
}



pub fn astar_t<S>(s: &S, start: (i32, i32), end: (i32, i32)) -> Option<RingBuf<(i32, i32)>> where
S: TwoDimSearchProblem
{
    let rspw = TwoDimSearchProblemWrapper {
        start: RefCell::new(Some(start)),
        end: end,
        rsp: s
    };

    astar(rspw)
}


