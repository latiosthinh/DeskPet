//! Spin Until Dizzy.

use crate::*;

pub struct Spin;

impl Trick for Spin {
    fn id(&self) -> &'static str {
        "spin"
    }
    fn title(&self) -> &'static str {
        "Spin Until Dizzy"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 3.0 && cx.once(1) {
            cx.say("whoa…", 1.5);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Spins through four facings, topples over with stars circling, then gets up.
        if t < 3.0 {
            look.frame = rotated(p.frame(eyes, Pose::Stand(0)), (t * 5.0) as usize % 4);
            look.props = vec![prop(rows(HERO_AXE), 15.0, 2.0)];
        } else if t < 6.0 {
            look.frame = rotated(p.frame(Eyes::Closed, Pose::Stand(0)), 1);
            look.y_offset = 2.0;
            look.free = vec![cx.placed(HERO_STAR, 6.0 + (6.0 * (t * 5.0).sin()).round(), 16.0)];
        } else {
            look.frame = p.frame(Eyes::Happy, Pose::Stand(0));
        }
        look
    }
}
