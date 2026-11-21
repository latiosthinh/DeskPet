//! Call the Fire Spirits.

use crate::*;

pub struct Spirits;

impl Trick for Spirits {
    fn id(&self) -> &'static str {
        "spirits"
    }
    fn title(&self) -> &'static str {
        "Call the Fire Spirits"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 0.3 && cx.once(1) {
            cx.say("rise, little flames!", 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Two little fire spirits spring up beside him, bob about, spit a few sparks and go out.
        look.frame = if t < 0.6 { Picture::marker_index(WITCHER_CAST) } else { p.frame(if t > SPIRITS_GO { Eyes::Happy } else { eyes }, Pose::Sit) };
        let dir = if cx.facing_left { -1.0 } else { 1.0 };
        let px = cx.pet.picture.as_ref().map_or(1.6, |p| p.px);
        for (k, x) in spirit_xs(cx, t).into_iter().enumerate() {
            if (SPIRITS_COME..SPIRITS_GO).contains(&t) {
                let flicker = cx.step_of(0.11 + 0.03 * k as f64);
                let bob = (3.0 * ((t * 5.0 + k as f64 * 1.7).sin() + 1.0)).round();
                let grid = mirrored(rows(WITCHER_SPIRIT[flicker]), cx.facing_left);
                look.free.push(spell_sprite(grid, x, bob, px * 1.4, false));
                // Now and then each one spits a spark ahead.
                let spit = (t - SPIRITS_COME - 1.2 - k as f64 * 0.6) % 2.2;
                if (0.0..0.6).contains(&spit) {
                    let sx = x + dir * (10.0 + 90.0 * spit);
                    look.free.push(spell_sprite(rows(&[".67.", "6776", "6776", ".67."]), sx, 12.0 + bob, px, false));
                }
            }
            for at in [SPIRITS_COME - 0.1, SPIRITS_GO - 0.1] {
                if (at..at + 0.45).contains(&t) {
                    look.free.push(free(rows(HERO_PUFF), x - 16.0, 0.0, PIXEL));
                }
            }
        }
        look
    }
}
