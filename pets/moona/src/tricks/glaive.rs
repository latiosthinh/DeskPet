//! Throw the Glaive.

use crate::*;

pub struct Glaive;

impl Trick for Glaive {
    fn id(&self) -> &'static str {
        "glaive"
    }
    fn title(&self) -> &'static str {
        "Throw the Glaive"
    }
    fn duration(&self) -> f64 {
        2.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        // Another click came in while the glaive was out: throw it again once it's back.
        if t >= self.duration() - 0.05 && cx.trick_again > 0 {
            cx.trick_again -= 1;
            cx.restart_trick();
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Throws her glaive like a boomerang: it spins out ahead of her, loops round and comes
        // back to her hand.
        if t < THROW_OUT {
            look.frame = Picture::marker_index(MOONA_WEAPON);
        } else if t < THROW_BACK {
            look.frame = Picture::marker_index(MOONA_BARE);
            // A fading trail, then the glaive itself.
            for (lag, faint) in [(0.07, true), (0.035, true), (0.0, false)] {
                let q = ((t - lag - THROW_OUT) / (THROW_BACK - THROW_OUT)).max(0.0);
                let (x, y) = glaive_at(cx, q);
                look.free.push(glaive_sprite(cx, x, y, (t - lag) * 17.0, faint));
            }
        } else {
            look.frame = p.frame(if t > THROW_BACK + 0.15 { Eyes::Happy } else { eyes }, Pose::Sit);
            if t < THROW_BACK + 0.25 {
                let (x, y) = glaive_at(cx, 0.0);
                look.free.push(free(rows(TWINKLE), x - 6.0, y - 6.0, PIXEL));
            }
        }
        look
    }
}
