//! Attract a Moth.

use crate::*;

pub struct Moth;

impl Trick for Moth {
    fn id(&self) -> &'static str {
        "moth"
    }
    fn title(&self) -> &'static str {
        "Attract a Moth"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 7.0 && cx.once(1) {
            cx.say("!", 0.8);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // A moth circles the glow, lands on its head, and the wisp hops in surprise.
        look.frame = p.frame(if t > 7.0 { Eyes::Happy } else { eyes }, Pose::Stand(0));
        if t < 5.0 {
            let dx = 8.0 + (14.0 * (t * 3.0).cos()).round();
            let up = 16.0 - (4.0 * (t * 3.0).sin()).round();
            look.free = vec![cx.placed(HERO_MOTH, dx, up)];
        } else {
            look.free = vec![cx.placed(HERO_MOTH, 7.0, if t > 7.0 { 17.0 } else { 14.0 })];
        }
        if t > 7.0 {
            look.y_offset = 3.0;
        }
        look
    }
}
