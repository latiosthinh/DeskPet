//! Run Down and Wind Up.

use crate::*;

pub struct Wind;

impl Trick for Wind {
    fn id(&self) -> &'static str {
        "wind"
    }
    fn title(&self) -> &'static str {
        "Run Down and Wind Up"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, dt: f64) {
        // Slows to a stop, waits for his key to turn, then hurries off happily.
        let dir = if cx.facing_left { -1.0 } else { 1.0 };
        if t < 3.0 {
            cx.pet_x += dir * 30.0 * (-t).exp() * dt;
        }
        if t > 7.0 {
            cx.pet_x += dir * 36.0 * dt;
        }
        if t > 5.0 && cx.once(1) {
            cx.say("click… click…", 1.8);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // Steps slow to a stop (step_trick moves him), the key on his back turns, then off he goes.
        if t < 3.0 {
            look.frame = p.frame(eyes, Pose::Stand((8.0 * (1.0 - (-t).exp())) as usize % 2));
        } else if t < 7.0 {
            look.frame = p.frame(Eyes::Closed, Pose::Stand(0));
        } else {
            look.frame = p.frame(Eyes::Happy, Pose::Stand((cx.clock / 0.15) as usize % 2));
        }
        let turning = t > 5.0 && t < 7.0 && beat > 0.0;
        look.back = vec![cx.placed(if turning { HERO_KEY_B } else { HERO_KEY_A }, -4.0, 4.0)];
        look
    }
}
