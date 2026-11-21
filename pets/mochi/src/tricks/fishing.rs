//! Go Fishing.

use crate::*;

pub struct Fishing;

impl Trick for Fishing {
    fn id(&self) -> &'static str {
        "fishing"
    }
    fn title(&self) -> &'static str {
        "Go Fishing"
    }
    fn duration(&self) -> f64 {
        11.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let blue = sys_color::BLUE;
        let p = t % 3.0;
        if p > 2.0 && cx.once(300 + (t / 3.0) as i32) {
            for _ in 0..6 {
                let at = pt(cx.center_x(21.0) - 4.0, 12.0);
                cx.spawn_full("•", blue, at, 9.0, (-30.0, 30.0), (30.0, 55.0), 0.7, -160.0);
            }
        }
        if p > 2.45 && cx.once(400 + (t / 3.0) as i32) {
            cx.say("!", 0.6);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        let ph = t % 3.0;
        look.frame = p.frame(if ph > 2.3 { Eyes::Happy } else { eyes }, Pose::Sit);
        look.props.push(prop(pool(10), 15.0, 0.0));
        look.props.push(prop(rows(ROD), 13.0, 3.0));
        if ph > 2.0 && ph < 2.9 {
            let q = (ph - 2.0) / 0.9;
            look.props
                .push(prop(rows(FISH), 18.0, 3.0 + 9.0 * (PI * q).sin()));
        }
        look
    }
}
