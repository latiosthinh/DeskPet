//! Read a Biting Book.

use crate::*;

pub struct Book;

impl Trick for Book {
    fn id(&self) -> &'static str {
        "book"
    }
    fn title(&self) -> &'static str {
        "Read a Biting Book"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 3.0 && cx.once(1) {
            cx.say("ow!", 1.0);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // A book floats up, snaps shut on his nose, then drops while he recoils.
        let biting = t > 3.0 && t < 5.0;
        look.frame = p.frame(if biting { Eyes::Closed } else { eyes }, Pose::Stand(0));
        let col = if t < 3.0 {
            16.0
        } else if biting {
            14.0
        } else {
            17.0
        }; // 14 while he recoils 2 px, so the book stays put
        let lift = if t < 3.0 {
            (4.0 + t).round()
        } else if t < 5.0 {
            6.0
        } else {
            (6.0 - (t - 5.0) * 3.0).max(0.0).round()
        };
        look.props = vec![prop(rows(HERO_BOOK), col, lift)];
        if biting {
            look.x_offset = -2.0;
        }
        look
    }
}
