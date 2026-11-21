//! Get Tied in a Knot.

use crate::*;

pub struct Knot;

impl Trick for Knot {
    fn id(&self) -> &'static str {
        "knot"
    }
    fn title(&self) -> &'static str {
        "Get Tied in a Knot"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 3.0 && cx.once(1) {
            cx.say("oops, knotted", 1.8);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // Winds a glowing loop, ends up with it knotted on its face, then shakes free.
        if t < 3.0 {
            look.frame = p.frame(eyes, Pose::Stand(0));
            look.free = vec![cx.placed(HERO_KNOT, 16.0 - (t * 3.0).round(), 5.0)];
        } else if t < 6.0 {
            look.frame = p.frame(Eyes::Closed, Pose::Stand(0));
            look.x_offset = beat;
            look.props = vec![prop(rows(HERO_KNOT), 7.0, 5.0)];
        } else {
            look.frame = p.frame(Eyes::Happy, Pose::Stand(0));
        }
        look
    }
}
