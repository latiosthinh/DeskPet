//! Grow Up.

use crate::*;

pub struct GrowUp;

impl Trick for GrowUp {
    fn id(&self) -> &'static str {
        "grow-up"
    }
    fn title(&self) -> &'static str {
        "Grow Up"
    }
    fn duration(&self) -> f64 {
        9.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if ((t > 0.5 && t < 3.0) || (t > 6.0 && t < 7.0)) && cx.every(1, 0.15) {
            let at = pt(cx.pet_center_x() + rand(-40.0, 34.0), rand(4.0, 70.0));
            cx.spawn_full("✦", sys_color::YELLOW, at, 10.0, (-4.0, 4.0), (4.0, 12.0), 0.8, 0.0);
        }
        if t > 3.0 && cx.once(1) {
            cx.say("so big!", 2.0);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        let scale = if t < 1.0 {
            1.0
        } else if t < 3.0 {
            1.0 + 0.125 * ((((t - 1.0) / 0.5) as i32 + 1).min(4)) as f64
        } else if t < 6.0 {
            1.5
        } else if t < 7.0 {
            1.5 - 0.125 * ((((t - 6.0) / 0.25) as i32 + 1).min(4)) as f64
        } else {
            1.0
        };
        look.scale = scale;
        look.frame = p.frame(if scale > 1.0 { Eyes::Happy } else { eyes }, Pose::Stand(0));
        look
    }
}
