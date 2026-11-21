//! Hop for Joy: the smallest kind of trick. It only changes how the pet looks over time.

use crate::*;

pub struct Hop;

impl Trick for Hop {
    fn id(&self) -> &'static str {
        "hop"
    }
    fn title(&self) -> &'static str {
        "Hop for Joy"
    }
    fn duration(&self) -> f64 {
        4.0
    }

    /// Things that happen: speech, particles, moving. `t` is seconds since the trick started.
    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if cx.once(1) {
            cx.say("yay!", 1.5);
        }
        // A sparkle each time it lands (every half second after the first).
        if t > 0.5 && cx.every(2, 0.5) {
            cx.spawn_above_head("✦", sys_color::YELLOW, 1, 1.0);
        }
    }

    /// How it looks at time `t`: start from `cx.look()` (sitting, blinking) and change it.
    fn look(&self, cx: &Stage, t: f64) -> Look {
        let mut look = cx.look();
        look.frame = cx.pet.frame(Eyes::Happy, Pose::Stand(0));
        // Up and down twice a second, in whole art pixels.
        look.y_offset = ((t * 2.0 * PI).sin().abs() * 4.0).round();
        look
    }
}
