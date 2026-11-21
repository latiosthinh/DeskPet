//! Play with Yarn.

use crate::*;

/// Whether Mochi is running after the yarn.
#[derive(Default)]
pub struct Yarn {
    chase_moving: bool,
}

impl Trick for Yarn {
    fn id(&self) -> &'static str {
        "yarn"
    }
    fn title(&self) -> &'static str {
        "Play with Yarn"
    }
    fn duration(&self) -> f64 {
        10.0
    }

    fn step(&mut self, cx: &mut Stage, t: f64, dt: f64) {
        let pink = sys_color::PINK;
        let ball = yarn_x(cx, t);
        let behind = if ball > cx.pet_center_x() { 22.0 } else { -22.0 };
        let target = (ball - PET_SIZE / 2.0 - behind).max(0.0).min(cx.width - PET_SIZE);
        self.chase_moving = !cx.move_toward(target, 50.0, dt);
        if !self.chase_moving {
            cx.facing_left = ball < cx.pet_center_x();
        }
        if t > 1.0 && cx.every(1, 2.4) {
            cx.spawn_above_head("♪", pink, 1, 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        look.frame = p.frame(Eyes::Happy, Pose::Stand(if self.chase_moving { cx.step_of(0.14) } else { 0 }));
        let x = yarn_x(cx, t);
        let rolling = (x / 6.0) as i64 % 2 == 0;
        look.free.push(free(rows(if rolling { YARN_A } else { YARN_B }), x - 8.0, 0.0, PIXEL));
        look
    }
}
