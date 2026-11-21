//! What it does while you type.

use crate::*;

pub struct Cooking;

impl Typing for Cooking {
    fn step(&mut self, cx: &mut Stage, _t: f64, _dt: f64) {
        if cx.every(1, 0.35) {
            let at = pt(cx.center_x(20.0) - 4.0, 8.0 * PIXEL);
            cx.spawn_full("~", sys_color::SECONDARY_LABEL, at, 11.0, (-3.0, 3.0), (10.0, 16.0), 0.8, 0.0);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // An egg in a pan over a little campfire, tossed now and then.
        let ph = t % 2.5;
        let mut egg = EGG_UP;
        let mut lift = 7.0;
        if ph > 1.6 && ph < 2.3 {
            lift = 7.0 + 6.0 * (PI * (ph - 1.6) / 0.7).sin();
            egg = if cx.alt(0.1) { EGG_UP } else { EGG_DOWN };
        }
        look.frame = p.frame(if ph > 2.3 { Eyes::Happy } else { eyes }, Pose::Stand(0));
        look.props.push(prop(rows(if cx.alt(0.25) { FIRE_A } else { FIRE_B }), 17.0, 0.0));
        look.props.push(prop(rows(PAN), 13.0, 5.0));
        look.props.push(prop(rows(egg), 18.0, lift));
        look
    }
}
