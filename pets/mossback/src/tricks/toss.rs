//! Toss a Rock, Then Nap.

use crate::*;

pub struct Toss;

impl Trick for Toss {
    fn id(&self) -> &'static str {
        "toss"
    }
    fn title(&self) -> &'static str {
        "Toss a Rock, Then Nap"
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Tosses a pebble in an arc, then naps standing up.
        if t < 3.0 {
            look.frame = p.frame(eyes, Pose::Stand(0));
            look.free = vec![cx.placed(HERO_ROCK, 14.0 + t * 5.0, (16.0 * (t / 3.0 * PI).sin()).round())];
        } else {
            look.frame = p.frame(Eyes::Closed, Pose::Stand(0));
            look.free = vec![cx.placed(HERO_ROCK, 29.0, 0.0), cx.placed(HERO_Z, 10.0, 17.0 + (2.0 * (t % 2.0)).round())];
        }
        look
    }
}
