//! Chase a Butterfly.

use crate::*;

/// Whether Sprout is running after the butterfly.
#[derive(Default)]
pub struct ChaseButterfly {
    chase_moving: bool,
}

impl Trick for ChaseButterfly {
    fn id(&self) -> &'static str {
        "chase-butterfly"
    }
    fn title(&self) -> &'static str {
        "Chase a Butterfly"
    }
    fn duration(&self) -> f64 {
        12.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, dt: f64) {
        let d = cx.duration();
        let b = butterfly_point(cx, t, d);
        let behind = if b.x > cx.pet_center_x() { 16.0 } else { -16.0 };
        let target = (b.x - PET_SIZE / 2.0 - behind).max(0.0).min(cx.width - PET_SIZE);
        self.chase_moving = !cx.move_toward(target, 55.0, dt);
        if !self.chase_moving {
            cx.facing_left = b.x < cx.pet_center_x();
        }
        if t > d - 1.5 && cx.once(1) {
            cx.say("bye!", 1.5);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let d = cx.duration();
        let p = &cx.pet;
        let mut look = cx.look();
        look.frame = p.frame(Eyes::Happy, Pose::Stand(if self.chase_moving { cx.step_of(0.14) } else { 0 }));
        let b = butterfly_point(cx, t, d);
        let art = if cx.alt(0.15) { BUTTERFLY_A } else { BUTTERFLY_B };
        look.free.push(free(rows(art), b.x - 7.5, b.y, 3.0));
        look
    }
}
