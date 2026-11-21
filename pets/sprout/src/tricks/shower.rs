//! Take a Shower.

use crate::*;

pub struct Shower;

impl Trick for Shower {
    fn id(&self) -> &'static str {
        "shower"
    }
    fn title(&self) -> &'static str {
        "Take a Shower"
    }
    fn duration(&self) -> f64 {
        9.0
    }

    fn step(&mut self, cx: &mut Stage, _t: f64, _dt: f64) {
        let blue = sys_color::BLUE;
        let pink = sys_color::PINK;
        let cloud_left = cx.pet_center_x() - 20.0;
        if cx.every(1, 0.06) {
            let at = pt(cloud_left + rand(4.0, 34.0), 17.0 * PIXEL - 6.0);
            cx.spawn_full("|", blue.with_alpha(0.8), at, 9.0, (0.0, 0.0), (-95.0, -80.0), 0.65, 0.0);
        }
        if cx.every(2, 1.4) {
            let at = pt(cx.pet_center_x() + 22.0, 40.0);
            cx.spawn("♪", pink, at, 13.0);
        }
    }

    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        look.frame = p.frame(Eyes::Happy, if cx.alt(0.4) { Pose::Stand(0) } else { Pose::Sit });
        look.free.push(free(rows(CLOUD), cx.pet_center_x() - 20.0, 17.0 * PIXEL, PIXEL));
        look
    }
}
