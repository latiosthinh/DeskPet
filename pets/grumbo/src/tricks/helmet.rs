//! Helmet Slips Down.

use crate::*;

pub struct Helmet;

impl Trick for Helmet {
    fn id(&self) -> &'static str {
        "helmet"
    }
    fn title(&self) -> &'static str {
        "Helmet Slips Down"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 1.0 && cx.once(1) {
            cx.say("who turned off the lights?", 2.5);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // The helmet drops over his eyes, he shuffles about blind, then pushes it back up.
        look.frame = p.frame(if t >= 6.0 { Eyes::Happy } else { eyes }, Pose::Stand(0));
        if t > 1.0 && t < 6.0 {
            look.x_offset = beat;
            look.props = vec![
                prop(rows(HERO_HELMET), 1.0, 3.0),
                prop(rows(HERO_AXE), 15.0, 0.0),
            ];
        }
        look
    }
}
