//! Weigh the Orbs.

use crate::*;

pub struct Weigh;

impl Trick for Weigh {
    fn id(&self) -> &'static str {
        "weigh"
    }
    fn title(&self) -> &'static str {
        "Weigh the Orbs"
    }
    fn duration(&self) -> f64 {
        3.8
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 0.4 && cx.once(1) {
            cx.say("hmm…", 1.6);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Turns to each orb in turn, orange, blue, pink, and round again.
        look.frame = if (0.2..3.4).contains(&t) {
            Picture::marker_index(WITCHER_SIDE + ((t - 0.2) / 0.55) as usize % 3)
        } else {
            p.frame(if t >= 3.4 { Eyes::Happy } else { eyes }, Pose::Sit)
        };
        look
    }
}
