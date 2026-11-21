//! Meteor.

use crate::*;

pub struct Meteor;

impl Trick for Meteor {
    fn id(&self) -> &'static str {
        "meteor"
    }
    fn title(&self) -> &'static str {
        "Meteor"
    }
    fn duration(&self) -> f64 {
        5.6
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 0.3 && cx.once(1) {
            cx.say("meteor!", 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        // Calls a burning rock down out of the sky; it crashes ahead of him and rolls on, burning.
        look.frame = if t < METEOR_LAND {
            Picture::marker_index(WITCHER_CAST)
        } else if t < METEOR_LAND + 0.6 {
            Picture::marker_index(WITCHER_AFTER)
        } else {
            p.frame(Eyes::Happy, Pose::Sit)
        };
        let dir = if cx.facing_left { -1.0 } else { 1.0 };
        let land = spell_x(cx, 60.0);
        let px = cx.pet.picture.as_ref().map_or(1.6, |p| p.px);
        if (METEOR_START..METEOR_LAND).contains(&t) {
            let q = (t - METEOR_START) / (METEOR_LAND - METEOR_START);
            let (x, y) = (
                land - dir * 90.0 * (1.0 - q),
                8.0 + (cx.height - 30.0) * (1.0 - q),
            );
            let grid = mirrored(meteor(t, true), cx.facing_left);
            look.free.push(spell_sprite(grid, x, y, px, cx.facing_left));
        }
        // Fires left burning where it struck (the biggest, and longest) and all along where
        // it rolled: each flickers, dies down to embers and goes out.
        for i in 0..=METEOR_FIRES {
            let s = i as f64 * METEOR_ROLL / METEOR_FIRES as f64;
            let passed = 1.0 - (1.0 - s / METEOR_ROLL).max(0.0).sqrt(); // inverse of the roll below
            let age = t - (METEOR_LAND + passed * (METEOR_END - METEOR_LAND));
            let stage = age / if i == 0 { 1.8 } else { 1.1 };
            let grid: &[&str] = if age < 0.0 {
                continue;
            } else if stage < 1.0 {
                if (cx.step_of(0.12 + 0.02 * (i % 3) as f64) + i) % 2 == 0 {
                    GROUND_FIRE_A
                } else {
                    GROUND_FIRE_B
                }
            } else if stage < 1.6 {
                GROUND_FIRE_LOW
            } else if stage < 2.0 {
                GROUND_EMBERS
            } else {
                continue;
            };
            let size = if i == 0 { px * 1.6 } else { px };
            look.back
                .push(spell_sprite(rows(grid), land + dir * s, 0.0, size, false));
        }
        if (METEOR_LAND..METEOR_END).contains(&t) {
            let q = (t - METEOR_LAND) / (METEOR_END - METEOR_LAND);
            let x = land + dir * METEOR_ROLL * (1.0 - (1.0 - q) * (1.0 - q));
            let grid = mirrored(meteor(t, false), cx.facing_left);
            look.free
                .push(spell_sprite(grid, x, 0.0, px, cx.facing_left));
            if t < METEOR_LAND + 0.5 {
                look.free
                    .push(free(rows(HERO_PUFF), land - 24.0, 0.0, PIXEL * 1.5));
                look.free
                    .push(free(rows(HERO_SPARK), land - 30.0, 18.0 + 40.0 * q, PIXEL));
                look.free
                    .push(free(rows(HERO_SPARK), land + 18.0, 24.0 + 40.0 * q, PIXEL));
            }
        }
        look
    }
}
