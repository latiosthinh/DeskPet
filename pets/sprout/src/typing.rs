//! What it does while you type.

use crate::*;

pub struct Digging;

impl Typing for Digging {
    fn step(&mut self, cx: &mut Stage, _t: f64, _dt: f64) {
        if (cx.clock / 0.3) as i64 % 2 == 1 && cx.every(1, 0.3) {
            for _ in 0..2 {
                let at = pt(cx.center_x(18.0) - 3.0, 8.0);
                let vx = cx.forward(-10.0, 40.0);
                cx.spawn_full(
                    "·",
                    sys_color::PLAIN_BROWN,
                    at,
                    16.0,
                    vx,
                    (30.0, 55.0),
                    0.7,
                    -150.0,
                );
            }
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        let down = !cx.alt(0.3);
        let level = (1 + (t / 2.0) as usize).min(3);
        look.frame = p.frame(eyes, Pose::Stand(if down { 1 } else { 0 }));
        look.props
            .push(prop(rows(&PILE[PILE.len() - level..]), 15.0, 0.0));
        look.props
            .push(prop(rows(SHOVEL), 13.0, if down { 1.0 } else { 3.0 }));
        look
    }
}
