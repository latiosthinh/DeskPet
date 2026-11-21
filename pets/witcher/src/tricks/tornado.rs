//! Tornado.

use crate::*;

pub struct Tornado;

impl Trick for Tornado {
    fn id(&self) -> &'static str {
        "tornado"
    }
    fn title(&self) -> &'static str {
        "Tornado"
    }
    fn duration(&self) -> f64 {
        3.7
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 0.3 && cx.once(1) {
            cx.say("tornado!", 1.4);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        // Sends a little whirlwind spinning off ahead of him along the ground.
        look.frame = if t < 0.5 {
            Picture::marker_index(WITCHER_THROW)
        } else if t < TORNADO_END {
            Picture::marker_index(WITCHER_AFTER)
        } else {
            p.frame(Eyes::Happy, Pose::Sit)
        };
        if (0.35..TORNADO_END).contains(&t) {
            let q = (t - 0.35) / (TORNADO_END - 0.35);
            let dir = if cx.facing_left { -1.0 } else { 1.0 };
            let start = cx.pet_center_x() + dir * 30.0;
            let x = start + (spell_x(cx, 280.0) - start) * q;
            let (px, tall) = cx
                .pet
                .picture
                .as_ref()
                .map_or((1.6, 56), |p| (p.px, p.height));
            let grid = tornado(t, tall, q > 0.88); // as tall as he is
            let w = grid[0].len() as f64 * px;
            look.free.push(FreeSprite {
                own_colors: true,
                ..free(grid, x - w / 2.0, 0.0, px)
            });
            if cx.alt(0.15) {
                look.free
                    .push(free(rows(HERO_PUFF), x - 16.0, 0.0, PIXEL / 2.0));
            }
        }
        look
    }
}
