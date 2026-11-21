//! Toast a Marshmallow.

use crate::*;

pub struct Campfire;

impl Trick for Campfire {
    fn id(&self) -> &'static str {
        "campfire"
    }
    fn title(&self) -> &'static str {
        "Toast a Marshmallow"
    }
    fn duration(&self) -> f64 {
        10.0
    }

    fn step(&mut self, cx: &mut Stage, _t: f64, _dt: f64) {
        let grey = sys_color::SECONDARY_LABEL;
        if cx.every(1, 0.45) {
            let at = pt(cx.center_x(18.0) - 4.0, 6.0 * PIXEL);
            cx.spawn_full("~", grey, at, 12.0, (-4.0, 4.0), (12.0, 20.0), 1.6, 0.0);
        }
        if cx.every(2, 0.3) {
            let at = pt(cx.center_x(18.0) - 3.0, 5.0 * PIXEL);
            cx.spawn_full(
                "·",
                sys_color::ORANGE,
                at,
                16.0,
                (-10.0, 10.0),
                (18.0, 32.0),
                0.8,
                0.0,
            );
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let d = cx.duration();
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        look.frame = p.frame(if t > d - 2.0 { Eyes::Happy } else { eyes }, Pose::Sit);
        look.props.push(prop(
            rows(if cx.alt(0.25) { FIRE_A } else { FIRE_B }),
            15.0,
            0.0,
        ));
        look.props.push(prop(stick(t > d * 0.5), 13.0, 3.0));
        look
    }
}
