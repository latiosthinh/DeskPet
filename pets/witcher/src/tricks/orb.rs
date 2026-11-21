//! Cast an Orb.

use crate::*;

pub struct Orb;

impl Trick for Orb {
    fn id(&self) -> &'static str {
        "orb"
    }
    fn title(&self) -> &'static str {
        "Cast an Orb"
    }
    fn duration(&self) -> f64 {
        1.8
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        // Holds out an orb, it bursts from his hand and flies off ahead, popping where it lands.
        let kind = (cx.mode_start * 7.0) as usize % 3; // a different orb from throw to throw
        look.frame = if t < ORB_LAUNCH {
            Picture::marker_index(WITCHER_THROW)
        } else if t < ORB_LAUNCH + 0.25 {
            Picture::marker_index(WITCHER_BURST)
        } else if t < ORB_LAND {
            Picture::marker_index(WITCHER_AFTER)
        } else {
            p.frame(Eyes::Happy, Pose::Sit)
        };
        if (ORB_LAUNCH..ORB_LAND).contains(&t) {
            for (lag, faint) in [(0.08, true), (0.04, true), (0.0, false)] {
                let q = ((t - lag - ORB_LAUNCH) / (ORB_LAND - ORB_LAUNCH)).max(0.0);
                let (x, y) = witcher_hand(cx, q);
                look.free.push(orb_sprite(cx, kind, x, y, faint));
            }
        }
        if (ORB_LAND..ORB_LAND + 0.4).contains(&t) {
            let (x, y) = witcher_hand(cx, 1.0);
            look.free.push(free(rows(HERO_PUFF), x - 16.0, (y - 14.0).max(0.0), PIXEL));
            look.free.push(free(rows(TWINKLE), x - 6.0, y - 6.0 + 20.0 * (t - ORB_LAND), PIXEL));
        }
        look
    }
}
