//! What it does while you type: swims laps in its pool.

use crate::*;

#[derive(Default)]
pub struct Swimming {
    pool: Pool,
}

impl Typing for Swimming {
    fn begin(&mut self, cx: &mut Stage) {
        self.pool.set_up(cx);
    }

    fn step(&mut self, cx: &mut Stage, _t: f64, dt: f64) {
        self.pool.swim(cx, dt);
    }

    fn look(&self, cx: &Stage, _t: f64) -> Look {
        self.pool.look(cx, cx.blink_eyes())
    }
}
