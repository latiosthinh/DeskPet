//! Hide in the Water.

use crate::*;

pub struct HideInWater;

impl Trick for HideInWater {
    fn id(&self) -> &'static str {
        "hide-in-water"
    }
    fn title(&self) -> &'static str {
        "Hide in the Water"
    }
    fn duration(&self) -> f64 {
        9.5
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let blue = sys_color::BLUE;
        if t > 0.8 && t < 3.5 && cx.every(1, 0.4) {
            let at = pt(cx.pet_center_x() + rand(-12.0, 6.0), 6.0);
            cx.spawn_full("°", blue, at, 13.0, (-3.0, 3.0), (14.0, 22.0), 1.0, 0.0);
        }
        if t > 5.2 && cx.once(1) {
            cx.facing_left = !cx.facing_left;
        }
        if t > 6.4 && cx.once(2) {
            cx.facing_left = !cx.facing_left;
        }
        if t > 7.9 && cx.once(3) {
            for _ in 0..10 {
                let at = pt(cx.pet_center_x() - 4.0, 10.0);
                cx.spawn_full("•", blue, at, 10.0, (-55.0, 55.0), (35.0, 75.0), 0.9, -170.0);
            }
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        // Sink, wait under water, peek out with just the eyes, then jump out with a splash.
        let y = if t < 0.8 {
            -12.0 * t / 0.8
        } else if t < 3.5 {
            -12.0
        } else if t < 4.3 {
            -12.0 + 8.0 * (t - 3.5) / 0.8
        } else if t < 7.5 {
            -4.0
        } else if t < 7.9 {
            -4.0 + 7.0 * (t - 7.5) / 0.4
        } else if t < 8.2 {
            3.0 - 3.0 * (t - 7.9) / 0.3
        } else {
            0.0
        };
        look.y_offset = y;
        look.frame = p.frame(if t > 7.5 { Eyes::Happy } else { eyes }, Pose::Stand(0));
        if t < 8.6 {
            look.props.push(prop(pool(20), -2.0, 0.0));
        }
        look
    }
}
