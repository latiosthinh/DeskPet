//! What it does while you type.

use crate::*;

pub struct Stamping;

impl Typing for Stamping {
    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let mut look = cx.look();
        // Grumbo stamps a tally sheet, one thump per beat.
        let beat = if cx.alt(0.2) { 0.0 } else { 1.0 };
        look.props.push(prop(rows(HERO_PAD), 14.0, 0.0));
        look.props
            .push(prop(rows(HERO_STAMP), 15.0, 4.0 + beat * 3.0));
        look
    }
}
