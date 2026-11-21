//! Go for a Swim.

use crate::*;

#[derive(Default)]
pub struct Swim {
    pool: Pool,
}

impl Trick for Swim {
    fn id(&self) -> &'static str {
        "swim"
    }
    fn title(&self) -> &'static str {
        "Go for a Swim"
    }
    fn duration(&self) -> f64 {
        10.0
    }

    fn start(&mut self, cx: &mut Stage) {
        self.pool.set_up(cx);
    }

    fn step(&mut self, cx: &mut Stage, _t: f64, dt: f64) {
        self.pool.swim(cx, dt);
    }

    fn look(&self, cx: &Stage, _t: f64) -> Look {
        self.pool.look(cx, Eyes::Open)
    }
}
