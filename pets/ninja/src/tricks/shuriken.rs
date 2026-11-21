//! Throw Shuriken.

use crate::*;

pub struct Shuriken;

impl Trick for Shuriken {
    fn id(&self) -> &'static str {
        "shuriken"
    }
    fn title(&self) -> &'static str {
        "Throw Shuriken"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > SHURIKEN_THROWS[0] && cx.once(1) {
            cx.say("thwip!", 1.0);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Flicks three shuriken one after another; each spins across and thunks into the target.
        look.frame = p.frame(if t > 4.0 { Eyes::Happy } else { eyes }, Pose::Stand(0));
        look.free.push(cx.placed(NINJA_TARGET, 25.0, 0.0));
        for (i, &at) in SHURIKEN_THROWS.iter().enumerate() {
            if t < at {
                continue;
            }
            let q = ((t - at) / 0.35).min(1.0);
            let (dx, up) = ([23.0, 24.0, 22.0][i], [6.0, 3.0, 4.0][i]);
            let spinning = q < 1.0;
            let star = if spinning && cx.alt(0.05) {
                NINJA_SHURIKEN_B
            } else {
                NINJA_SHURIKEN_A
            };
            look.free
                .push(cx.placed(star, 14.0 + (dx - 14.0) * q, (8.0 + (up - 8.0) * q).round()));
            if t < at + 0.15 {
                look.x_offset = 1.0; // the flick
            }
        }
        look
    }
}
