//! Blow Bubbles.

use crate::*;

pub struct Bubbles;

impl Trick for Bubbles {
    fn id(&self) -> &'static str {
        "bubbles"
    }
    fn title(&self) -> &'static str {
        "Blow Bubbles"
    }
    fn duration(&self) -> f64 {
        7.0
    }

    fn step(&mut self, cx: &mut Stage, _t: f64, _dt: f64) {
        if cx.every(1, 0.45) {
            let at = pt(cx.center_x(15.0), 5.5 * PIXEL);
            let vx = cx.forward(8.0, 22.0);
            cx.spawn_full(
                "○",
                sys_color::TEAL,
                at,
                rand(9.0, 15.0),
                vx,
                (12.0, 24.0),
                2.4,
                0.0,
            );
        }
    }

    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        look.frame = p.frame(
            if cx.alt(0.45) {
                Eyes::Happy
            } else {
                Eyes::Open
            },
            Pose::Sit,
        );
        look
    }
}
