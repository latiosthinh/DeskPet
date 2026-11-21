//! Eat Ice Cream.

use crate::*;

pub struct IceCream;

impl Trick for IceCream {
    fn id(&self) -> &'static str {
        "ice-cream"
    }
    fn title(&self) -> &'static str {
        "Eat Ice Cream"
    }
    fn duration(&self) -> f64 {
        9.0
    }

    fn step(&mut self, cx: &mut Stage, _t: f64, _dt: f64) {
        let pink = sys_color::PINK;
        if cx.every(1, 1.8) {
            cx.spawn_above_head("♥", pink, 1, 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let d = cx.duration();
        let p = &cx.pet;
        let mut look = cx.look();
        look.frame = p.frame(if cx.alt(0.5) { Eyes::Happy } else { Eyes::Open }, Pose::Sit);
        look.props.push(prop(ice_cream(1.0 - t / d), 14.0, 3.0));
        look
    }
}
