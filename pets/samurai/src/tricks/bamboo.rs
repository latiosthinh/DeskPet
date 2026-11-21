//! Cut the Bamboo.

use crate::*;

pub struct Bamboo;

impl Trick for Bamboo {
    fn id(&self) -> &'static str {
        "bamboo"
    }
    fn title(&self) -> &'static str {
        "Cut the Bamboo"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > BAMBOO_CUT + 0.3 && cx.once(1) {
            cx.say("slice!", 1.2);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Faces a bamboo stalk, one quick slash, and the top slides off and drops.
        look.frame = p.frame(if t > 5.5 { Eyes::Happy } else { eyes }, Pose::Stand(0));
        let slash = BAMBOO_CUT;
        if (slash - 0.15..slash + 0.15).contains(&t) {
            look.x_offset = 3.0; // the lunge
        }
        look.free.push(cx.placed(SAMURAI_BAMBOO_BOTTOM, 21.0, 0.0));
        let q = ((t - slash - 0.4) / 0.7).clamp(0.0, 1.0);
        look.free.push(cx.placed(
            SAMURAI_BAMBOO_TOP,
            21.0 + (4.0 * q).round(),
            (9.0 * (1.0 - q)).round(),
        ));
        if (slash..slash + 0.25).contains(&t) {
            look.free.push(cx.placed(SAMURAI_SLASH, 12.0, 9.0));
        }
        if (slash..5.5).contains(&t) {
            look.props = vec![prop(rows(SAMURAI_KATANA), 12.0, 5.0)];
        }
        look
    }
}
