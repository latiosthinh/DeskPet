//! Give You a Flower.

use crate::*;

pub struct GiveFlower;

impl Trick for GiveFlower {
    fn id(&self) -> &'static str {
        "give-flower"
    }
    fn title(&self) -> &'static str {
        "Give You a Flower"
    }
    fn duration(&self) -> f64 {
        7.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let pink = sys_color::PINK;
        if cx.once(1) {
            cx.say_with("for you!", 4.0, Some(bubble_icons::TULIP));
        }
        if t > 1.0 && cx.every(1, 1.1) {
            cx.spawn_above_head("♥", pink, 1, 1.4);
        }
    }

    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        look.frame = p.frame(Eyes::Happy, Pose::Stand(0));
        look.props.push(prop(rows(FLOWER), 14.0, if cx.alt(0.6) { 4.0 } else { 5.0 }));
        look
    }
}
