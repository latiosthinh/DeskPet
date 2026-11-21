//! Change Colour.

use crate::*;

pub struct ColorShift;

impl Trick for ColorShift {
    fn id(&self) -> &'static str {
        "color_shift"
    }
    fn title(&self) -> &'static str {
        "Change Colour"
    }

    fn step(&mut self, cx: &mut Stage, _t: f64, _dt: f64) {
        if cx.every(1, 0.22) {
            let at = pt(cx.pet_center_x() + rand(-32.0, 26.0), rand(6.0, 50.0));
            cx.spawn_full(
                "✦",
                sys_color::SPARKLES[rand_index(6)],
                at,
                10.0,
                (-4.0, 4.0),
                (4.0, 10.0),
                0.8,
                0.0,
            );
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let d = cx.duration();
        let p = &cx.pet;
        let mut look = cx.look();
        look.palette = hue_shifted(&p.palette, t / d, &['c', 'd', 'k']);
        look.frame = p.frame(Eyes::Happy, Pose::Stand(if cx.alt(0.5) { 0 } else { 1 }));
        look
    }
}
