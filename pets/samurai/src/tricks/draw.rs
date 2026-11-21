//! Quick Draw.

use crate::*;

pub struct Draw;

impl Trick for Draw {
    fn id(&self) -> &'static str {
        "draw"
    }
    fn title(&self) -> &'static str {
        "Quick Draw"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > DRAW_CUT && cx.once(1) {
            cx.say("…", 1.2);
        }
        if t > 5.0 && cx.once(2) {
            cx.say("*click*", 1.2);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Meditates while a leaf drifts down, then one flash of the blade and the leaf falls in two.
        let cut = DRAW_CUT;
        look.frame = p.frame(
            if t < cut {
                Eyes::Closed
            } else if t > 5.0 {
                Eyes::Happy
            } else {
                eyes
            },
            Pose::Stand(0),
        );
        if t < cut {
            let q = t / cut;
            look.free.push(cx.placed(SAMURAI_LEAF, 20.0 + (1.5 * (t * 3.0).sin()).round(), (17.0 - 11.0 * q).round()));
        } else {
            let q = ((t - cut) / 1.2).min(1.0);
            let up = (6.0 * (1.0 - q)).round();
            look.free.push(cx.placed(SAMURAI_LEAF_L, 20.0 - (2.0 * q).round(), up));
            look.free.push(cx.placed(SAMURAI_LEAF_R, 22.0 + (2.0 * q).round(), up));
            if t < cut + 0.25 {
                look.free.push(cx.placed(SAMURAI_SLASH, 13.0, 7.0));
            }
            if t < 5.0 {
                look.props = vec![prop(rows(SAMURAI_KATANA), 12.0, 5.0)];
            }
        }
        look
    }
}
