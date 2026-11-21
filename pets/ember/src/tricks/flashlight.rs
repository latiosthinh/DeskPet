//! Explore with a Flashlight.

use crate::*;

pub struct Flashlight;

impl Trick for Flashlight {
    fn id(&self) -> &'static str {
        "flashlight"
    }
    fn title(&self) -> &'static str {
        "Explore with a Flashlight"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let grey = sys_color::SECONDARY_LABEL;
        cx.facing_left = ((t / 2.7) as i32 % 2 == 1) != cx.trick_start_facing;
        if cx.every(1, 2.2) {
            cx.spawn_above_head("?", grey, 1, 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        look.frame = p.frame(eyes, Pose::Stand(0));
        look.props.push(prop(rows(FLASHLIGHT), 14.0, 5.0));
        let origin = pt(
            cx.center_x(18.0) + if cx.facing_left { -2.0 } else { 2.0 },
            6.5 * PIXEL,
        );
        look.beam = Some((origin, 0.05 + 0.22 * (t * 1.6).sin()));
        look
    }
}
