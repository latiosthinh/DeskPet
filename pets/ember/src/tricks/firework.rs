//! Light Fireworks.

use crate::*;

/// Remembers which burst it has already set off.
#[derive(Default)]
pub struct Firework {
    burst_cycle: i32,
}

impl Trick for Firework {
    fn id(&self) -> &'static str {
        "firework"
    }
    fn title(&self) -> &'static str {
        "Light Fireworks"
    }

    fn start(&mut self, _cx: &mut Stage) {
        self.burst_cycle = -1;
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let cycle = (t / 3.5) as i32;
        let p = t % 3.5;
        if p < 1.2 && cx.every(1, 0.08) {
            let at = pt(cx.center_x(17.0) - 4.0, 22.0);
            cx.spawn_full(
                "✦",
                sys_color::ORANGE,
                at,
                9.0,
                (-10.0, 10.0),
                (4.0, 14.0),
                0.45,
                0.0,
            );
        }
        if p >= 2.0 && self.burst_cycle != cycle {
            self.burst_cycle = cycle;
            let at = pt(cx.center_x(17.0), 98.0);
            firework(cx, at);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let mut look = cx.look();
        let ph = t % 3.5;
        look.frame = p.frame(if ph > 2.0 { Eyes::Happy } else { eyes }, Pose::Sit);
        if ph < 1.2 {
            look.props.push(prop(rows(ROCKET), 16.0, 0.0));
        } else if ph < 2.0 {
            look.props
                .push(prop(rows(ROCKET), 16.0, (ph - 1.2) / 0.8 * 20.0));
        }
        look
    }
}
