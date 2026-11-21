//! Chase a Laser Dot.

use crate::*;

/// Where the dot is, and whether Mochi is running after it.
#[derive(Default)]
pub struct Laser {
    dot_x: f64,
    chase_moving: bool,
}

impl Trick for Laser {
    fn id(&self) -> &'static str {
        "laser"
    }
    fn title(&self) -> &'static str {
        "Chase a Laser Dot"
    }
    fn duration(&self) -> f64 {
        10.0
    }

    fn start(&mut self, cx: &mut Stage) {
        self.dot_x = cx.pet_center_x();
    }

    fn step(&mut self, cx: &mut Stage, t: f64, dt: f64) {
        // The dot jumps every 1.6 s; Mochi crouches, wiggles, then pounces on it.
        let cycle = (t / 1.6) as i32;
        if cx.once(200 + cycle) && cycle > 0 {
            self.dot_x = rand(12.0, (cx.width - 12.0).max(13.0));
        }
        if t % 1.6 > 0.55 {
            let off = if self.dot_x > cx.pet_center_x() {
                10.0
            } else {
                -10.0
            };
            let target = (self.dot_x - PET_SIZE / 2.0 - off)
                .max(0.0)
                .min(cx.width - PET_SIZE);
            self.chase_moving = !cx.move_toward(target, 120.0, dt);
        } else {
            self.chase_moving = false;
            cx.facing_left = self.dot_x < cx.pet_center_x();
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        let ph = t % 1.6;
        if self.chase_moving {
            look.frame = p.frame(Eyes::Open, Pose::Stand(0));
            look.y_offset = 4.0 * (PI * ((ph - 0.55) / 0.5).min(1.0)).sin();
        } else {
            look.frame = p.frame(
                Eyes::Open,
                if cx.alt(0.12) {
                    Pose::Sit
                } else {
                    Pose::Stand(0)
                },
            ); // wiggles before pouncing
        }
        look.free
            .push(free(rows(LASER_DOT), self.dot_x - 2.5, 0.0, 5.0));
        look
    }
}
