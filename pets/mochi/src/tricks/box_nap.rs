//! Nap in a Box.

use crate::*;

pub struct BoxNap;

impl Trick for BoxNap {
    fn id(&self) -> &'static str {
        "box_nap"
    }
    fn title(&self) -> &'static str {
        "Nap in a Box"
    }
    fn duration(&self) -> f64 {
        10.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let d = cx.duration();
        let grey = sys_color::SECONDARY_LABEL;
        if t > 1.4 && t < d - 1.4 && cx.every(1, 1.3) {
            cx.spawn_above_head("z", grey, 1, 2.2);
        }
        if t > d - 1.2 && cx.once(1) {
            cx.say("!", 1.0);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let d = cx.duration();
        let p = &cx.pet;
        let mut look = cx.look();
        // Hop into the box, nap with just the head showing, then pop up.
        let y = if t < 0.8 {
            2.0 + 8.0 * (PI * t / 0.8).sin()
        } else if t > d - 1.2 {
            3.0
        } else {
            2.0
        };
        look.y_offset = y;
        look.frame = p.frame(if t > 1.2 && t < d - 1.2 { Eyes::Closed } else { Eyes::Happy }, Pose::Sit);
        look.props.push(prop(rows(BOX), -1.0, 0.0));
        look
    }
}
