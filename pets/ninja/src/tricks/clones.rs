//! Clone Trick.

use crate::*;

pub struct Clones;

impl Trick for Clones {
    fn id(&self) -> &'static str {
        "clones"
    }
    fn title(&self) -> &'static str {
        "Clone Trick"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > CLONE_IN + 0.6 && cx.once(1) {
            cx.say("which one is me?", 1.8);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Two copies of him pop up either side; all three hop in turn, then the copies vanish.
        let turn = (CLONE_IN + 1.0..CLONE_OUT - 0.3)
            .contains(&t)
            .then(|| ((t / 0.25) as usize) % 3);
        let f = p.frame(
            if turn.is_some() { Eyes::Happy } else { eyes },
            Pose::Stand(0),
        );
        look.frame = f;
        if (CLONE_IN..CLONE_OUT).contains(&t) {
            for (k, dx) in [(0, -15.0), (2, 15.0)] {
                look.back
                    .push(cx.frame_copy(&f, dx, if turn == Some(k) { 3.0 } else { 0.0 }));
            }
            if turn == Some(1) {
                look.y_offset = 3.0;
            }
        }
        for at in [CLONE_IN - 0.1, CLONE_OUT - 0.1] {
            if (at..at + 0.5).contains(&t) {
                let rise = ((t - at) * 6.0).round();
                look.free.push(cx.placed(HERO_PUFF, -11.0, 3.0 + rise));
                look.free.push(cx.placed(HERO_PUFF, 19.0, 3.0 + rise));
            }
        }
        look
    }
}
