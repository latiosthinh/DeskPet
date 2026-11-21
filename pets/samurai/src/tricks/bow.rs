//! Bow.

use crate::*;

pub struct Bow;

impl Trick for Bow {
    fn id(&self) -> &'static str {
        "bow"
    }
    fn title(&self) -> &'static str {
        "Bow"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 1.0 && cx.once(1) {
            cx.say("honour.", 1.6);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Bows low while cherry petals drift down round him.
        let bowing = (0.6..3.6).contains(&t);
        let f = p.frame(
            if bowing {
                Eyes::Closed
            } else if t > 3.6 {
                Eyes::Happy
            } else {
                eyes
            },
            Pose::Stand(0),
        );
        look.frame = if bowing { lean(f, 2) } else { f };
        for i in 0..7 {
            let start = 0.4 + i as f64 * 0.9;
            let up = 18.0 - (t - start) * 5.0;
            if t < start || up < 0.0 {
                continue;
            }
            let dx = -6.0 + i as f64 * 4.5 + (1.5 * (t * 2.0 + i as f64).sin()).round();
            look.free.push(cx.placed(SAMURAI_PETAL, dx, up.round()));
        }
        look
    }
}
