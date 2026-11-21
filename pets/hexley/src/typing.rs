//! What it does while you type.

use crate::*;

pub struct SpellNotes;

impl Typing for SpellNotes {
    fn look(&self, cx: &Stage, t: f64) -> Look {
        let mut look = cx.look();
        // Hexley copies spell notes; the writing floats off the page.
        let beat = if cx.alt(0.2) { 0.0 } else { 1.0 };
        look.props.push(prop(rows(HERO_BOOK), 14.0, 0.0));
        look.props.push(prop(rows(HERO_WAND), 14.0, 5.0 + beat));
        look.props.push(prop(rows(HERO_SPARK), 18.0, (7.0 + 5.0 * (t % 2.0)).round()));
        look
    }
}
