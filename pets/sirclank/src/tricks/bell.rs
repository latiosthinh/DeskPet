//! Tea-Time Alarm.

use crate::*;

pub struct Bell;

impl Trick for Bell {
    fn id(&self) -> &'static str {
        "bell"
    }
    fn title(&self) -> &'static str {
        "Tea-Time Alarm"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 3.0 && cx.once(1) {
            cx.say("tea time!", 2.2);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // An alarm bell rises out of his helmet and rattles; he jumps.
        let rattling = t > 3.0 && t < 6.0;
        look.frame = p.frame(eyes, Pose::Stand(0));
        if t < 7.0 {
            let col = 7.0 + if rattling { beat } else { 0.0 };
            look.props = vec![prop(rows(HERO_BELL), col, (14.0 + t.min(3.0)).round())];
        }
        if rattling {
            look.y_offset = beat * 2.0;
        }
        look
    }
}
