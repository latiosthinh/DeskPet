//! Spit Water.

use crate::*;

pub struct SpitWater;

impl Trick for SpitWater {
    fn id(&self) -> &'static str {
        "spit-water"
    }
    fn title(&self) -> &'static str {
        "Spit Water"
    }
    fn duration(&self) -> f64 {
        6.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let blue = sys_color::BLUE;
        if t % 0.9 < 0.1 && cx.once(100 + (t / 0.9) as i32) {
            for _ in 0..7 {
                let at = pt(cx.center_x(15.0), 5.5 * PIXEL);
                let vx = cx.forward(55.0, 95.0);
                cx.spawn_full("•", blue, at, 9.0, vx, (25.0, 55.0), 1.1, -170.0);
            }
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        look.frame = p.frame(if t % 0.9 < 0.2 { Eyes::Happy } else { Eyes::Open }, Pose::Sit);
        look
    }
}
