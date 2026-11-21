//! Smoke Bomb.

use crate::*;

pub struct Smoke;

impl Trick for Smoke {
    fn id(&self) -> &'static str {
        "smoke"
    }
    fn title(&self) -> &'static str {
        "Smoke Bomb"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > SMOKE_GONE && cx.once(1) {
            cx.say("poof!", 1.0);
        }
        // Gone: he turns up somewhere else in the pen.
        if t > (SMOKE_GONE + SMOKE_BACK) / 2.0 && cx.once(2) {
            let from = cx.pet_x;
            cx.pet_x = cx.random_target(from, f64::MAX);
            cx.facing_left = cx.pet_x > from;
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // Holds up a smoke bomb, vanishes in a cloud of smoke, and pops up somewhere else.
        let gone = (SMOKE_GONE..SMOKE_BACK).contains(&t);
        look.frame = if gone {
            [[b'.'; 16]; 16]
        } else {
            p.frame(
                if t > SMOKE_BACK { Eyes::Happy } else { eyes },
                Pose::Stand(0),
            )
        };
        if t < SMOKE_GONE - 0.2 {
            look.props = vec![prop(rows(NINJA_BOMB), 14.0, 6.0 + beat)];
        }
        for at in [SMOKE_GONE - 0.2, SMOKE_BACK - 0.2] {
            if (at..at + 0.9).contains(&t) {
                let grow = ((t - at) * 4.0).round();
                look.free.push(cx.placed(HERO_PUFF, -grow, 0.0));
                look.free.push(cx.placed(HERO_PUFF, 8.0 + grow, 2.0));
                look.free.push(cx.placed(HERO_PUFF, 4.0, 6.0 + grow));
            }
        }
        look
    }
}
