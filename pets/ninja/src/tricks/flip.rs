//! Backflips.

use crate::*;

pub struct Flip;

impl Trick for Flip {
    fn id(&self) -> &'static str {
        "flip"
    }
    fn title(&self) -> &'static str {
        "Backflips"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > FLIPS[0] && cx.once(1) {
            cx.say("hup!", 0.8);
        }
        if t > FLIPS[1] && cx.once(2) {
            cx.say("hup!", 0.8);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Crouches and does a backflip, lands in a puff of dust, and does another.
        look.frame = p.frame(if t > FLIPS[1] + FLIP_TIME { Eyes::Happy } else { eyes }, Pose::Stand(0));
        for &at in &FLIPS {
            if (at - 0.3..at).contains(&t) {
                look.y_offset = -1.0;
            } else if (at..at + FLIP_TIME).contains(&t) {
                let q = (t - at) / FLIP_TIME;
                look.y_offset = (12.0 * (PI * q).sin()).round();
                look.frame = rotated(look.frame, (4 - (q * 4.0) as usize) % 4);
            } else if (at + FLIP_TIME..at + FLIP_TIME + 0.35).contains(&t) {
                look.free.push(cx.placed(HERO_PUFF, -5.0, 0.0));
                look.free.push(cx.placed(HERO_PUFF, 13.0, 0.0));
            }
        }
        look
    }
}
