//! Meditate.

use crate::*;

pub struct Meditate;

impl Trick for Meditate {
    fn id(&self) -> &'static str {
        "meditate"
    }
    fn title(&self) -> &'static str {
        "Meditate"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 1.5 && cx.once(1) {
            cx.say("ommm…", 2.0);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        // Sits with his eyes shut, floats up, and leaves circle round him.
        look.frame = p.frame(if t > 6.5 { Eyes::Happy } else { Eyes::Closed }, Pose::Sit);
        let rise = if t < 1.0 {
            0.0
        } else if t < 6.0 {
            (7.0 * (t - 1.0).min(1.0) + (t * 2.0).sin()).round()
        } else {
            (14.0 * (6.5 - t).max(0.0)).round()
        };
        look.y_offset = rise;
        if (1.2..6.2).contains(&t) {
            for k in 0..4 {
                let a = t * 1.8 + k as f64 * PI / 2.0;
                let leaf = cx.placed(
                    NINJA_LEAF,
                    (6.0 + 11.0 * a.cos()).round(),
                    (rise + 5.0 + 3.0 * a.sin()).round(),
                );
                if a.sin() > 0.0 {
                    look.back.push(leaf);
                } else {
                    look.free.push(leaf);
                }
            }
        }
        look
    }
}
