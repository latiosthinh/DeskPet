//! What it does while you type.

use crate::*;

pub struct ReadingGlasses;

impl Typing for ReadingGlasses {
    fn look(&self, cx: &Stage, t: f64) -> Look {
        let mut look = cx.look();
        look.props
            .push(prop(props::book(t % 3.0 < 0.25), 13.0, cx.book_lift()));
        look.free.push(glasses_sprite(cx));
        look
    }
}
