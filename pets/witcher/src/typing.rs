//! What it does while you type.

use crate::*;

pub struct Orbs;

impl Typing for Orbs {
    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let look = cx.look();

        look
    }
}
