//! Pounce.

use crate::*;

pub struct Pounce;

impl Trick for Pounce {
    fn id(&self) -> &'static str {
        "pounce"
    }
    fn title(&self) -> &'static str {
        "Pounce"
    }
    fn duration(&self) -> f64 {
        2.6
    }

    fn step(&mut self, cx: &mut Stage, t: f64, dt: f64) {
        // Leaps forward.
        if (POUNCE_UP..POUNCE_LAND).contains(&t) {
            let dir = if cx.trick_start_facing { -1.0 } else { 1.0 };
            cx.pet_x += dir * 60.0 * dt;
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Gathers, springs forward in an arc, lands with a puff of dust.
        if (POUNCE_UP..POUNCE_LAND).contains(&t) {
            let q = (t - POUNCE_UP) / (POUNCE_LAND - POUNCE_UP);
            look.frame = Picture::marker_index(MOONA_JUMP);
            look.y_offset = (8.0 * 4.0 * q * (1.0 - q)).round();
        } else {
            look.frame = p.frame(
                if t >= POUNCE_LAND + 0.4 {
                    Eyes::Happy
                } else {
                    eyes
                },
                Pose::Sit,
            );
        }
        if (POUNCE_LAND..POUNCE_LAND + 0.45).contains(&t) {
            look.free = vec![
                cx.placed(HERO_PUFF, -14.0, 0.0),
                cx.placed(HERO_PUFF, 22.0, 0.0),
            ];
        }
        look
    }
}
