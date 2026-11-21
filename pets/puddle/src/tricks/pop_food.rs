//! Pop Out a Snack.

use crate::*;

/// Which snack pops out.
#[derive(Default)]
pub struct PopFood {
    food_kind: usize,
}

impl Trick for PopFood {
    fn id(&self) -> &'static str {
        "pop_food"
    }
    fn title(&self) -> &'static str {
        "Pop Out a Snack"
    }
    fn duration(&self) -> f64 {
        7.0
    }

    fn start(&mut self, _cx: &mut Stage) {
        self.food_kind = rand_index(FOOD.len());
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 1.2 && cx.once(1) {
            cx.say("!", 0.8);
        }
        if t > 2.0 && cx.once(2) {
            cx.say("a snack for you!", 3.0);
            let at = pt(cx.center_x(20.0) - 4.0, 12.0);
            cx.spawn_full(
                "✦",
                sys_color::YELLOW,
                at,
                11.0,
                (-10.0, 10.0),
                (22.0, 36.0),
                0.9,
                0.0,
            );
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        if t < 1.2 {
            look.frame = p.frame(Eyes::Closed, Pose::Stand(cx.step_of(0.12)));
        } else {
            look.frame = p.frame(Eyes::Happy, Pose::Sit);
            let q = ((t - 1.2) / 0.8).min(1.0);
            look.props.push(prop(
                rows(FOOD[self.food_kind]),
                6.0 + 13.0 * q,
                (1.0 - q) * 10.0 + 12.0 * (PI * q).sin(),
            ));
        }
        look
    }
}
