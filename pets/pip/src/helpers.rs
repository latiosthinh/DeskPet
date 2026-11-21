//! The paddling pool Pip swims in, for the Swim trick and while you type.

use crate::*;

/// A pool on the floor, set out under Pip; Pip swims from one end to the other.
#[derive(Default)]
pub struct Pool {
    left: f64,
    width: f64,
}

impl Pool {
    /// Sets the pool out under Pip, kept inside the pen, and heads for its far end.
    pub fn set_up(&mut self, cx: &mut Stage) {
        self.width = 30.0 * PIXEL;
        self.left = (cx.pet_center_x() - self.width / 2.0).max(0.0).min((cx.width - self.width).max(0.0));
        cx.target_x = self.range(cx).1;
    }

    /// The left-edge positions Pip swims between.
    fn range(&self, cx: &Stage) -> (f64, f64) {
        let lo = (self.left - 8.0).max(0.0);
        let hi = lo.max((cx.width - PET_SIZE).min(self.left + self.width - PET_SIZE + 8.0));
        (lo, hi)
    }

    /// Swims back and forth, leaving ripples.
    pub fn swim(&self, cx: &mut Stage, dt: f64) {
        let (lo, hi) = self.range(cx);
        if cx.move_toward(cx.target_x, 22.0, dt) {
            cx.target_x = if (cx.target_x - lo).abs() < 1.0 { hi } else { lo };
        }
        if cx.every(1, 0.35) {
            let at = pt(cx.pet_center_x() + if cx.facing_left { 22.0 } else { -30.0 }, 6.0);
            cx.spawn_full("~", sys_color::BLUE, at, 12.0, (-4.0, 4.0), (2.0, 6.0), 0.8, 0.0);
        }
    }

    /// Pip in the water, with the pool drawn round it.
    pub fn look(&self, cx: &Stage, eyes: Eyes) -> Look {
        let mut look = cx.look();
        look.y_offset = -3.0;
        look.frame = cx.pet.frame(eyes, Pose::Stand(cx.step_of(0.25)));
        look.free.push(free(pool((self.width / PIXEL) as usize), self.left, 0.0, PIXEL));
        look
    }
}
