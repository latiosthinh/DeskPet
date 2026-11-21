//! Sneeze a Spring.

use crate::*;

pub struct Spring;

impl Trick for Spring {
    fn id(&self) -> &'static str {
        "spring"
    }
    fn title(&self) -> &'static str {
        "Sneeze a Spring"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 2.0 && cx.once(1) {
            cx.say("achoo!", 1.2);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // A spring pops out of his hatch, wobbles, and gets reeled back in.
        look.frame = p.frame(eyes, Pose::Stand(0));
        if t > 2.0 && t < 7.0 {
            let out = ((if t < 5.0 { t - 2.0 } else { 7.0 - t }) * 2.0).round();
            look.props = vec![prop(rows(HERO_SPRING), 12.0 + out + if t < 3.0 { 2.0 } else { 0.0 }, 3.0)];
        }
        if t > 2.0 && t < 3.0 {
            look.x_offset = -2.0;
        }
        look
    }
}
