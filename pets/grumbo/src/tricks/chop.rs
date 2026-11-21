//! Chop a Log (Try To).

use crate::*;

pub struct Chop;

impl Trick for Chop {
    fn id(&self) -> &'static str {
        "chop"
    }
    fn title(&self) -> &'static str {
        "Chop a Log (Try To)"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 4.0 && cx.once(1) {
            cx.say("boing!", 1.0);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // Lifts the axe, strikes the log, and the axe bounces straight back up. The log is fine.
        look.frame = p.frame(eyes, Pose::Stand(0));
        let lift = if t < 2.0 {
            10.0
        } else if t < 4.0 {
            2.0
        } else if t < 6.0 {
            (2.0 + 10.0 * ((t - 4.0) / 2.0 * PI).sin()).round()
        } else {
            3.0
        };
        look.props = vec![
            prop(rows(HERO_LOG), 17.0, 0.0),
            prop(rows(HERO_AXE), 15.0, lift),
        ];
        if t > 4.0 && t < 6.0 {
            look.y_offset = -beat;
        }
        look
    }
}
