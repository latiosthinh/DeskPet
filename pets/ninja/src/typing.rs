//! What it does while you type.

use crate::*;

pub struct Scroll;

impl Typing for Scroll {
    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let mut look = cx.look();
        // Kage writes secret notes on a scroll, the brush dabbing in time with the keys.
        let beat = if cx.alt(0.2) { 0.0 } else { 1.0 };
        look.props.push(prop(rows(NINJA_SCROLL), 13.0, 0.0));
        look.props.push(prop(rows(NINJA_BRUSH), 15.0 + beat, 3.0 + beat));
        look
    }
}
