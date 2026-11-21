//! Vanish.

use crate::*;

pub struct Vanish;

impl Trick for Vanish {
    fn id(&self) -> &'static str {
        "vanish"
    }
    fn title(&self) -> &'static str {
        "Vanish"
    }
    fn duration(&self) -> f64 {
        4.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        // Gone: he turns up somewhere else in the pen.
        if t > VANISH_MOVE && cx.once(1) {
            let from = cx.pet_x;
            cx.pet_x = cx.random_target(from, f64::MAX);
            cx.facing_left = cx.pet_x > from;
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        // Fades into a ghost and away, then fades back in somewhere else.
        let fade = if t < 0.4 {
            look.frame = Picture::marker_index(WITCHER_CAST);
            1.0
        } else if t < VANISH_BACK + 0.8 {
            look.frame = Picture::marker_index(WITCHER_GHOST);
            if t < VANISH_GONE {
                1.0 - ((t - 0.6) / (VANISH_GONE - 0.6)).clamp(0.0, 1.0)
            } else if t < VANISH_BACK {
                0.0
            } else {
                (t - VANISH_BACK) / 0.8
            }
        } else {
            look.frame = p.frame(Eyes::Happy, Pose::Sit);
            1.0
        };
        if fade < 1.0 {
            look.palette = look.palette.faded(fade);
        }
        // A wisp where he goes and where he comes back.
        for at in [VANISH_GONE - 0.3, VANISH_BACK] {
            if (at..at + 0.5).contains(&t) {
                let c = cx.pet_center_x();
                look.free
                    .push(free(rows(TWINKLE), c - 6.0, 40.0 + 30.0 * (t - at), PIXEL));
            }
        }
        look
    }
}
