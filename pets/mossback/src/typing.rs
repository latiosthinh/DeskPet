//! What it does while you type.

use crate::*;

pub struct Pebbles;

impl Typing for Pebbles {
    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let mut look = cx.look();
        // Mossback sorts pebbles into a little row.
        let beat = if cx.alt(0.2) { 0.0 } else { 1.0 };
        look.props.push(prop(rows(HERO_PEBBLES), 14.0, 0.0));
        look.props
            .push(prop(rows(HERO_ROCK), 14.0 + 5.0 * beat, 3.0 + 2.0 * beat));
        look
    }
}
