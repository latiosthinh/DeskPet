//! Sun Strike.

use crate::*;

pub struct SunStrike;

impl Trick for SunStrike {
    fn id(&self) -> &'static str {
        "sun_strike"
    }
    fn title(&self) -> &'static str {
        "Sun Strike"
    }
    fn duration(&self) -> f64 {
        2.6
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 0.3 && cx.once(1) {
            cx.say("sun strike!", 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // Raises his hands, then a shaft of sunlight burns down on a spot ahead of him.
        let x = spell_x(cx, 70.0);
        look.frame = if t < SUN_CALL {
            Picture::marker_index(WITCHER_CAST)
        } else if t < SUN_END {
            Picture::marker_index(WITCHER_THROW)
        } else {
            p.frame(Eyes::Happy, Pose::Sit)
        };
        if (SUN_CALL..SUN_HIT).contains(&t) {
            // A thin warning glimmer first.
            look.free
                .push(light_shaft(cx, x, 4, 1.0, ['.', '3', '3', '2', '2'], false));
        } else if (SUN_HIT..SUN_END).contains(&t) {
            let reach = ((t - SUN_HIT) / 0.12).min(1.0);
            look.free.push(light_shaft(
                cx,
                x,
                16 + 2 * beat as usize,
                reach,
                ['.', '3', '2', '1', '@'],
                t > SUN_END - 0.3,
            ));
            if t < SUN_HIT + 0.5 {
                look.free.push(free(rows(HERO_PUFF), x - 16.0, 0.0, PIXEL));
                look.free.push(free(
                    rows(TWINKLE),
                    x - 22.0,
                    10.0 + 30.0 * (t - SUN_HIT),
                    PIXEL,
                ));
                look.free.push(free(
                    rows(TWINKLE),
                    x + 10.0,
                    16.0 + 30.0 * (t - SUN_HIT),
                    PIXEL,
                ));
            }
        }
        look
    }
}
