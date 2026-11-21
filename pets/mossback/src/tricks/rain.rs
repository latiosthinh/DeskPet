//! Enjoy a Little Rain.

use crate::*;

pub struct Rain;

impl Trick for Rain {
    fn id(&self) -> &'static str {
        "rain"
    }
    fn title(&self) -> &'static str {
        "Enjoy a Little Rain"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let pink = sys_color::PINK;
        if t > 6.0 && cx.every(1, 1.2) {
            cx.spawn_above_head("♥", pink, 1, 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // A little drizzle of his own; the moss drinks it up and brightens, then a happy shake.
        if t < 6.0 {
            look.frame = p.frame(eyes, Pose::Stand(0));
            let fall1 = ((t * 9.0) % 8.0).round();
            let fall2 = ((t * 9.0 + 3.0) % 8.0).round();
            look.free = vec![cx.placed(HERO_RAIN, 4.0, 19.0 - fall1), cx.placed(HERO_RAIN, 10.0, 19.0 - fall2)];
            if beat > 0.0 {
                look.palette.set('m', Some(hex(0xa8bc77)));
            }
        } else {
            look.frame = p.frame(Eyes::Happy, Pose::Stand(0));
            look.x_offset = beat;
        }
        look
    }
}
