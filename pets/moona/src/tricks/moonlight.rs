//! Call the Moonlight.

use crate::*;

pub struct Moonlight;

impl Trick for Moonlight {
    fn id(&self) -> &'static str {
        "moonlight"
    }
    fn title(&self) -> &'static str {
        "Call the Moonlight"
    }
    fn duration(&self) -> f64 {
        4.4
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 0.4 && cx.once(1) {
            cx.say("moonlight!", 1.6);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // Raises her glaive and calls down the moon: a broad beam of light falls on her, and
        // narrow beams strike the ground all round, one after another.
        let raised = (MOON_UP..MOON_DOWN).contains(&t);
        look.frame = if raised {
            Picture::marker_index(MOONA_STORM)
        } else {
            p.frame(if t >= MOON_DOWN { Eyes::Happy } else { eyes }, Pose::Sit)
        };
        if (MOON_UP..MOON_DOWN + 0.2).contains(&t) {
            let reach = ((t - MOON_UP) / 0.3).min(1.0);
            let width = 28 + 2 * beat as usize;
            look.back
                .push(moonbeam(cx, cx.pet_center_x(), width, reach, t > MOON_DOWN));
        }
        for i in 0..MOONBEAMS {
            let age = t - (MOON_UP + 0.3 + i as f64 * 0.25);
            if !(0.0..0.55).contains(&age) {
                continue;
            }
            let x = moonbeam_x(cx, i);
            look.free
                .push(moonbeam(cx, x, 6, (age / 0.12).min(1.0), age > 0.4));
            if (0.12..0.4).contains(&age) {
                look.free.push(free(rows(TWINKLE), x - 6.0, 0.0, PIXEL));
                look.free
                    .push(free(rows(HERO_PUFF), x - 8.0, 0.0, PIXEL / 2.0));
            }
        }
        look
    }
}
