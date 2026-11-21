//! Moon Blast.

use crate::*;

pub struct MoonBlast;

impl Trick for MoonBlast {
    fn id(&self) -> &'static str {
        "moon-blast"
    }
    fn title(&self) -> &'static str {
        "Moon Blast"
    }
    fn duration(&self) -> f64 {
        2.6
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 0.3 && cx.once(1) {
            cx.say("moon blast!", 1.4);
        }
        // The moon slash blows dust and pebbles away ahead of it as it goes, and sends a
        // last spray flying where it breaks up.
        let dir = if cx.facing_left { -1.0 } else { 1.0 };
        let push = |a: f64, b: f64| if dir > 0.0 { (a, b) } else { (-b, -a) };
        if (BLAST_SHOOT..BLAST_HIT).contains(&t) && cx.every(2, 0.03) {
            let at = pt(slash_x(cx, t) + dir * 8.0, 2.0);
            cx.spawn_full("·", sys_color::BROWN.with_alpha(0.85), at, 20.0, push(60.0, 170.0), (20.0, 80.0), 0.8, -180.0);
            if rand(0.0, 1.0) < 0.45 {
                cx.spawn_full("•", sys_color::SECONDARY_LABEL, at, 9.0, push(80.0, 200.0), (40.0, 110.0), 0.9, -260.0);
            }
        }
        if t > BLAST_HIT && cx.once(3) {
            let at = pt(slash_x(cx, BLAST_HIT), 4.0);
            for k in 0..12 {
                let (text, size) = if k % 3 == 0 { ("•", 9.0) } else { ("·", 20.0) };
                cx.spawn_full(text, sys_color::BROWN.with_alpha(0.85), at, size, push(40.0, 220.0), (40.0, 150.0), 1.0, -260.0);
            }
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        // A crescent of moonlight gathers in his outstretched hand, then slashes straight out
        // ahead of him (blowing the dust away as it goes, see step_trick) and breaks up.
        look.frame = if t < 0.3 {
            Picture::marker_index(WITCHER_CAST)
        } else if t < BLAST_HIT + 0.2 {
            Picture::marker_index(WITCHER_THROW)
        } else {
            p.frame(Eyes::Happy, Pose::Sit)
        };
        let px = cx.pet.picture.as_ref().map_or(1.6, |p| p.px);
        let (hand_x, hand_y) = witcher_hand(cx, 0.0);
        let flip = cx.facing_left;
        let slash = |size: f64, faint: bool, cx: f64| {
            let grid = mirrored(moon_slash(size, faint), flip);
            let half = grid.len() as f64 * px / 2.0;
            spell_sprite(grid, cx, (hand_y - half).max(0.0), px, false)
        };
        if (0.3..BLAST_SHOOT).contains(&t) {
            look.free.push(slash(0.3 + 0.5 * ((t - 0.3) / (BLAST_SHOOT - 0.3)), false, hand_x));
        } else if (BLAST_SHOOT..BLAST_HIT).contains(&t) {
            // Flies level, with fading copies trailing behind it.
            for (lag, faint) in [(0.08, true), (0.04, true), (0.0, false)] {
                look.free.push(slash(1.0, faint, slash_x(cx, t - lag)));
            }
        } else if (BLAST_HIT..BLAST_HIT + 0.4).contains(&t) {
            let q = (t - BLAST_HIT) / 0.4;
            look.free.push(slash(1.0 + 0.3 * q, true, slash_x(cx, BLAST_HIT)));
            look.free.push(free(rows(TWINKLE), slash_x(cx, BLAST_HIT) - 6.0, hand_y + 30.0 * q, PIXEL));
        }
        look
    }
}
