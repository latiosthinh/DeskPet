//! Triplicate.

use crate::*;

pub struct Triplicate;

impl Trick for Triplicate {
    fn id(&self) -> &'static str {
        "triplicate"
    }
    fn title(&self) -> &'static str {
        "Triplicate"
    }
    fn duration(&self) -> f64 {
        5.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 0.4 && cx.once(1) {
            cx.say("Manta style!", 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Two exact copies of her pop up on either side. All three send their glaives
        // whirling round themselves for a while, catch them, and the copies vanish.
        let spinning = (TRI_SPIN..TRI_CATCH).contains(&t);
        let frame = if spinning { MOONA_BARE } else { 2 }; // 2: standing still, glaive in hand
        look.frame = if spinning {
            Picture::marker_index(MOONA_BARE)
        } else {
            p.frame(if t >= TRI_CATCH { Eyes::Happy } else { eyes }, Pose::Sit)
        };
        let copies = copy_centers(cx);
        if (TRI_APPEAR..TRI_VANISH).contains(&t) {
            for (k, &x) in copies.iter().enumerate() {
                let orbit = spinning.then(|| whirl(cx, x, t, k as f64 + 1.0));
                if let Some((behind, _)) = &orbit {
                    look.back.extend(behind.iter().map(|g| glaive_copy(cx, g)));
                }
                look.back.push(picture_copy(cx, frame, x));
                if let Some((_, front)) = &orbit {
                    look.back.extend(front.iter().map(|g| glaive_copy(cx, g)));
                }
            }
        }
        if spinning {
            let (behind, front) = whirl(cx, cx.pet_center_x(), t, 0.0);
            look.back.extend(behind.iter().map(|g| glaive_copy(cx, g)));
            look.free.extend(front.iter().map(|g| glaive_copy(cx, g)));
        }
        // A puff where each copy appears and where it goes.
        for at in [TRI_APPEAR - 0.1, TRI_VANISH - 0.1] {
            if (at..at + 0.4).contains(&t) {
                for &x in &copies {
                    look.free.push(free(rows(HERO_PUFF), x - 16.0, 8.0, PIXEL));
                    look.free
                        .push(free(rows(TWINKLE), x - 6.0, 46.0 + 20.0 * (t - at), PIXEL));
                }
            }
        }
        look
    }
}
