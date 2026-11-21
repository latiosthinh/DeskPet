//! Catch a Fly.

use crate::*;

pub struct Fly;

impl Trick for Fly {
    fn id(&self) -> &'static str {
        "fly"
    }
    fn title(&self) -> &'static str {
        "Catch a Fly"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > FLY_CATCH && cx.once(1) {
            cx.say("got it.", 1.4);
        }
        if t > FLY_FREE && cx.once(2) {
            cx.say("off you go.", 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // A fly buzzes round his head; he waits with his eyes shut, snaps it out of the air
        // with chopsticks, then lets it go.
        let calm = (1.2..FLY_CATCH).contains(&t);
        look.frame = p.frame(
            if calm {
                Eyes::Closed
            } else if t > FLY_FREE {
                Eyes::Happy
            } else {
                eyes
            },
            Pose::Stand(0),
        );
        let fly = if cx.alt(0.04) { SAMURAI_FLY_A } else { SAMURAI_FLY_B };
        if t < FLY_CATCH {
            let (dx, up) = (9.0 + 7.0 * (t * 3.1).sin(), 15.0 + 3.0 * (t * 5.3).sin());
            look.free.push(cx.placed(fly, dx.round(), up.round()));
        } else if t < FLY_FREE {
            look.free.push(cx.placed(SAMURAI_CHOPSTICKS, 14.0, 6.0));
            look.free.push(cx.placed(fly, 18.0, 10.0));
            if t < FLY_CATCH + 0.15 {
                look.x_offset = 1.0;
            }
        } else {
            let q = t - FLY_FREE;
            look.free.push(cx.placed(fly, (18.0 + 6.0 * q).round(), (10.0 + 8.0 * q + (q * 9.0).sin()).round()));
            if q < 0.8 {
                look.free.push(cx.placed(SAMURAI_CHOPSTICKS, 14.0, 6.0));
            }
        }
        look
    }
}
