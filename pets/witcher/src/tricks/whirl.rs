//! Whirl the Orbs.

use crate::*;

pub struct Whirl;

impl Trick for Whirl {
    fn id(&self) -> &'static str {
        "whirl"
    }
    fn title(&self) -> &'static str {
        "Whirl the Orbs"
    }
    fn duration(&self) -> f64 {
        3.2
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > ORB_WHIRL_START && cx.once(1) {
            cx.say("orbs, unite!", 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        // Draws the orbs round him faster and faster, then sets them off in a flash.
        look.frame = if t < ORB_WHIRL_START {
            Picture::marker_index(WITCHER_CAST)
        } else if t < ORB_WHIRL_FLASH {
            Picture::marker_index(WITCHER_WHIRL + cx.step_of(0.12))
        } else if t < ORB_WHIRL_FLASH + 0.5 {
            Picture::marker_index(WITCHER_BURST)
        } else if t < ORB_WHIRL_FLASH + 0.8 {
            Picture::marker_index(WITCHER_AFTER)
        } else {
            p.frame(Eyes::Happy, Pose::Sit)
        };
        if (ORB_WHIRL_FLASH..ORB_WHIRL_FLASH + 0.6).contains(&t) {
            let c = cx.pet_center_x();
            let up = 20.0 * (t - ORB_WHIRL_FLASH);
            for (dx, dy) in [(-44.0, 40.0), (40.0, 52.0), (-30.0, 70.0), (50.0, 24.0)] {
                look.free.push(free(rows(TWINKLE), c + dx, dy + up, PIXEL));
            }
        }
        look
    }
}
