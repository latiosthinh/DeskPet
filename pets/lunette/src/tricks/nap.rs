//! Nap in a Crescent Moon.

use crate::*;

pub struct Nap;

impl Trick for Nap {
    fn id(&self) -> &'static str {
        "nap"
    }
    fn title(&self) -> &'static str {
        "Nap in a Crescent Moon"
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        // Tucks her ears down and naps on a crescent-moon cushion.
        let napping = (1.0..7.0).contains(&t);
        let nap_eyes = if t < 1.0 {
            Eyes::Open
        } else if napping {
            Eyes::Closed
        } else {
            Eyes::Happy
        };
        let mut frame = p.frame(nap_eyes, Pose::Sit);
        if napping {
            for (row, ears) in frame.iter_mut().zip(LUNETTE_TUCKED_EARS) {
                row.copy_from_slice(ears.as_bytes());
            }
        }
        look.frame = frame;
        look.y_offset = if napping { 1.0 } else { 2.0 }; // resting on the cushion
        look.back = vec![cx.placed(LUNETTE_CUSHION, -3.0, 0.0)];
        if napping {
            let drift = (2.0 * (t % 2.0)).round();
            look.free = vec![cx.placed(HERO_Z, 17.0, 18.0 + drift)];
            if t > 3.0 {
                look.free.push(cx.placed(HERO_Z, 23.0, 22.0 + drift));
            }
        }
        look
    }
}
