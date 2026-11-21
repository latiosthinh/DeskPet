//! Plant Itself.

use crate::*;

pub struct PlantSelf;

impl Trick for PlantSelf {
    fn id(&self) -> &'static str {
        "plant_self"
    }
    fn title(&self) -> &'static str {
        "Plant Itself"
    }
    fn duration(&self) -> f64 {
        10.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if (t < 1.0 || (t > 7.5 && t < 8.3)) && cx.every(1, 0.08) {
            let at = pt(cx.pet_center_x() + rand(-26.0, 20.0), 8.0);
            cx.spawn_full(
                "·",
                sys_color::PLAIN_BROWN,
                at,
                16.0,
                (-40.0, 40.0),
                (30.0, 60.0),
                0.7,
                -150.0,
            );
        }
        if t > 8.0 && cx.once(1) {
            cx.say("!", 1.0);
        }
        if t > 8.3 && t < 9.3 {
            cx.facing_left = (t / 0.2) as i32 % 2 == 0; // shakes off the dirt
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        // Dig, sink until only the leaves show, nap, then pop out and shake off.
        let y = if t < 1.0 {
            0.0
        } else if t < 2.5 {
            -8.0 * (t - 1.0) / 1.5
        } else if t < 7.5 {
            -8.0
        } else if t < 8.0 {
            -8.0 + 10.0 * (t - 7.5) / 0.5
        } else if t < 8.3 {
            2.0 - 2.0 * (t - 8.0) / 0.3
        } else {
            0.0
        };
        look.y_offset = y;
        let e = if t > 2.5 && t < 7.5 {
            Eyes::Closed
        } else {
            Eyes::Open
        };
        look.frame = p.frame(e, Pose::Stand(if t < 1.0 { cx.step_of(0.15) } else { 0 }));
        if t < 9.2 {
            look.props.push(prop(rows(SOIL), -2.0, 0.0));
        }
        look
    }
}
