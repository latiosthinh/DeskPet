//! Pick a Flower: a trick that remembers something (where it walked to) and holds a prop.

use crate::*;

/// Fields keep what the trick needs between frames. The same value is used every time the trick
/// plays, so `start` sets them up afresh.
#[derive(Default)]
pub struct Flower {
    walking: bool,
}

impl Trick for Flower {
    fn id(&self) -> &'static str {
        "flower"
    }
    fn title(&self) -> &'static str {
        "Pick a Flower"
    }
    fn duration(&self) -> f64 {
        6.0
    }

    fn start(&mut self, _cx: &mut Stage) {
        self.walking = true;
    }

    fn step(&mut self, cx: &mut Stage, t: f64, dt: f64) {
        // Walk to the middle of the pen first, then offer the flower.
        let middle = (cx.min_pet_x() + cx.max_pet_x()) / 2.0;
        self.walking = t < 3.0 && !cx.move_toward(middle, 40.0, dt);
        if t > 3.0 && cx.once(1) {
            cx.say("for you!", 2.0);
            cx.spawn_above_head("♥", sys_color::PINK, 2, 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let mut look = cx.look();
        let step = if self.walking { cx.step_of(0.15) } else { 0 };
        look.frame = cx.pet.frame(if t > 3.0 { Eyes::Happy } else { cx.blink_eyes() }, Pose::Stand(step));
        // A prop held at sprite column 13, lifted 4 art pixels off the ground.
        look.props.push(prop(rows(FLOWER), 13.0, 4.0));
        look
    }
}
