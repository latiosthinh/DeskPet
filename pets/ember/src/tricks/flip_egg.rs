//! Flip an Egg.

use crate::*;

pub struct FlipEgg;

impl Trick for FlipEgg {
    fn id(&self) -> &'static str {
        "flip-egg"
    }
    fn title(&self) -> &'static str {
        "Flip an Egg"
    }
    fn duration(&self) -> f64 {
        9.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let grey = sys_color::SECONDARY_LABEL;
        if t % 2.0 < 0.7 && cx.every(1, 0.3) {
            let at = pt(cx.center_x(19.0) - 4.0, 8.0 * PIXEL);
            cx.spawn_full("~", grey, at, 11.0, (-3.0, 3.0), (10.0, 16.0), 0.8, 0.0);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        let ph = t % 2.0;
        let mut egg = EGG_UP;
        let mut lift = 7.0;
        if (0.7..1.6).contains(&ph) {
            lift = 7.0 + 10.0 * (PI * (ph - 0.7) / 0.9).sin();
            egg = if cx.alt(0.1) { EGG_UP } else { EGG_DOWN };
        }
        look.frame = p.frame(if ph >= 1.6 { Eyes::Happy } else { eyes }, Pose::Stand(0));
        look.props.push(prop(rows(PAN), 13.0, 5.0));
        look.props.push(prop(rows(egg), 18.0, lift));
        look
    }
}
