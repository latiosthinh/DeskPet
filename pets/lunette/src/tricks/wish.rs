//! Wish on a Shooting Star.

use crate::*;

pub struct Wish;

impl Trick for Wish {
    fn id(&self) -> &'static str {
        "wish"
    }
    fn title(&self) -> &'static str {
        "Wish on a Shooting Star"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 4.0 && cx.once(1) {
            cx.say("a little wish…", 2.0);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // Gazes at the stars, a shooting star crosses, she closes her eyes and wishes.
        let wish_eyes = if t >= 6.0 {
            Eyes::Happy
        } else if t >= 4.0 {
            Eyes::Closed
        } else {
            Eyes::Open
        };
        look.frame = p.frame(wish_eyes, Pose::Sit);
        look.free = vec![
            cx.placed(HERO_STAR, -3.0, 23.0),
            cx.placed(HERO_STAR, 22.0, 26.0),
        ];
        if (2.0..4.0).contains(&t) {
            look.free.push(cx.placed(
                LUNETTE_COMET,
                -8.0 + (t - 2.0) * 22.0,
                (27.0 - (t - 2.0) * 4.5).round(),
            ));
        }
        if t >= 4.0 && beat > 0.0 {
            look.free.push(cx.placed(HERO_SPARK, 17.0, 17.0));
            look.free.push(cx.placed(HERO_SPARK, 5.0, 22.0));
        }
        look
    }
}
