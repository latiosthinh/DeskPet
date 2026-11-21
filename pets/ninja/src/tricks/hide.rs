//! Hide in a Bush.

use crate::*;

pub struct Hide;

impl Trick for Hide {
    fn id(&self) -> &'static str {
        "hide"
    }
    fn title(&self) -> &'static str {
        "Hide in a Bush"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 1.5 && cx.once(1) {
            cx.say("…", 1.2);
        }
        if t > HIDE_POP && cx.once(2) {
            cx.say("surprise!", 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Ducks into a bush, peeks out left and right, then jumps out.
        look.frame = p.frame(if t > HIDE_POP { Eyes::Happy } else { eyes }, Pose::Stand(0));
        if (0.8..HIDE_POP).contains(&t) {
            look.y_offset = -2.0;
            if (2.5..3.2).contains(&t) || (4.5..5.2).contains(&t) {
                look.x_offset = 1.0; // a peek
            }
        } else if (HIDE_POP..HIDE_POP + 0.5).contains(&t) {
            look.y_offset = 4.0;
        }
        if t > 0.6 {
            look.free.push(cx.placed(NINJA_BUSH, 2.0, 0.0));
        }
        look
    }
}
