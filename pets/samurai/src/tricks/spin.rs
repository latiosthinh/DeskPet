//! Whirlwind Slash.

use crate::*;

pub struct Spin;

impl Trick for Spin {
    fn id(&self) -> &'static str {
        "spin"
    }
    fn title(&self) -> &'static str {
        "Whirlwind Slash"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > SPIN.0 && cx.once(1) {
            cx.say("hyaa!", 1.0);
        }
        if (SPIN.0..SPIN.1).contains(&t) && cx.every(2, 0.12) {
            cx.facing_left = !cx.facing_left;
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Spins on the spot with his katana out while a ring of steel flashes round him.
        let spinning = (SPIN.0..SPIN.1).contains(&t);
        look.frame = p.frame(if t > SPIN.1 + 0.5 { Eyes::Happy } else { eyes }, Pose::Stand(0));
        if (SPIN.0 - 0.4..SPIN.1 + 1.0).contains(&t) {
            look.props = vec![prop(rows(SAMURAI_KATANA), 12.0, 5.0)];
        }
        if spinning {
            look.y_offset = if cx.alt(0.12) { 1.0 } else { 0.0 };
            let ring = cx.placed(SAMURAI_RING, -2.0, 3.0);
            if cx.alt(0.06) {
                look.free.push(ring);
            } else {
                look.back.push(ring);
            }
        }
        look
    }
}
