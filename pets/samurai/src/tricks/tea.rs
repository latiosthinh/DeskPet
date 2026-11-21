//! Tea Ceremony.

use crate::*;

pub struct Tea;

impl Trick for Tea {
    fn id(&self) -> &'static str {
        "tea"
    }
    fn title(&self) -> &'static str {
        "Tea Ceremony"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > TEA_SIP.1 && cx.once(1) {
            cx.say("ahh.", 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Kneels with a bowl of tea, steam curling up; sips it slowly with his eyes shut.
        let sipping = (TEA_SIP.0..TEA_SIP.1).contains(&t);
        look.frame = p.frame(
            if sipping {
                Eyes::Closed
            } else if t > TEA_SIP.1 {
                Eyes::Happy
            } else {
                eyes
            },
            Pose::Sit,
        );
        if (0.4..6.5).contains(&t) {
            let (col, lift) = if sipping { (11.0, 6.0) } else { (12.0, 2.0) };
            look.props = vec![prop(rows(SAMURAI_TEA), col, lift)];
            if !sipping {
                for k in 0..2 {
                    let q = (t * 0.8 + k as f64 * 0.5) % 1.0;
                    let wisp = if (q * 4.0) as usize % 2 == 0 {
                        SAMURAI_STEAM_A
                    } else {
                        SAMURAI_STEAM_B
                    };
                    look.free
                        .push(cx.placed(wisp, 13.0 + k as f64 * 2.0, (4.0 + q * 5.0).round()));
                }
            }
        }
        look
    }
}
