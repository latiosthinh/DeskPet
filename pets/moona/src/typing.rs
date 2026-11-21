//! What it does while you type.

use crate::*;

pub struct Moonlight;

impl Typing for Moonlight {
    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let mut look = cx.look();
        // Moona waits on her panther while a little moon glows in front of it, twinkling as you type.
        let beat = if cx.alt(0.2) { 0.0 } else { 1.0 };
        look.props.push(prop(rows(MOONA_MOON), 13.0, 15.0 + beat));
        if beat > 0.0 {
            look.props.push(prop(rows(TWINKLE), 12.0, 21.0));
        }
        look
    }
}
