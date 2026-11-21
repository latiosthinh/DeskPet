//! What it does while you type.

use crate::*;

pub struct Mortar;

impl Typing for Mortar {
    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let mut look = cx.look();
        // Lunette pounds rice cakes in time with the keys.
        let beat = if cx.alt(0.2) { 0.0 } else { 1.0 };
        look.props.push(prop(rows(LUNETTE_MORTAR), 14.0, 0.0));
        look.props.push(prop(rows(LUNETTE_MALLET), 14.0, 3.0 + beat * 4.0));
        look
    }
}
