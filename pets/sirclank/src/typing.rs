//! What it does while you type.

use crate::*;

pub struct Checklist;

impl Typing for Checklist {
    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let mut look = cx.look();
        // Sir Clank punches holes in a checklist.
        let beat = if cx.alt(0.2) { 0.0 } else { 1.0 };
        look.props.push(prop(rows(HERO_PAD), 14.0, 0.0));
        look.props.push(prop(rows(HERO_PAW), 15.0, 6.0 - 2.0 * beat));
        look
    }
}
