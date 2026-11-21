//! Split in Two and Play.

use crate::*;

/// Where the twin stands (its centre x).
#[derive(Default)]
pub struct Split {
    twin_center_x: f64,
}

impl Trick for Split {
    fn id(&self) -> &'static str {
        "twin"
    }
    fn title(&self) -> &'static str {
        "Split in Two and Play"
    }
    fn duration(&self) -> f64 {
        11.0
    }

    fn start(&mut self, cx: &mut Stage) {
        let room_right = cx.width - (cx.pet_x + PET_SIZE);
        let room_left = cx.pet_x;
        let side = if room_right >= room_left { 1.0 } else { -1.0 };
        self.twin_center_x = (cx.pet_center_x() + side * 68.0)
            .max(PET_SIZE / 2.0)
            .min(cx.width - PET_SIZE / 2.0);
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let d = cx.duration();
        let pink = sys_color::PINK;
        cx.facing_left = self.twin_center_x < cx.pet_center_x();
        if t > 0.6 && cx.once(1) {
            cx.say("pop!", 1.0);
        }
        if t > 1.4 && cx.every(1, 1.1) {
            let at = pt(
                (cx.pet_center_x() + self.twin_center_x) / 2.0 - 4.0,
                PET_SIZE - 8.0,
            );
            cx.spawn("♪", pink, at, 13.0);
        }
        if t > d - 0.6 && cx.once(2) {
            cx.say("pop!", 1.0);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let d = cx.duration();
        let p = &cx.pet;
        let mut look = cx.look();
        let (start, end) = (0.6, d - 1.2);
        if t < start {
            look.frame = p.frame(Eyes::Closed, Pose::Sit);
            return look;
        }
        let hop = ((t - start) * 2.0 * PI / 0.8).sin();
        look.frame = p.frame(Eyes::Happy, Pose::Stand(0));
        look.y_offset = hop.max(0.0) * 3.0;
        let mut scale = 1.0;
        let mut twin_x = self.twin_center_x;
        if t < start + 0.5 {
            scale = 0.4 + 0.6 * ((t - start) / 0.5);
            twin_x = lerp(cx.pet_center_x(), self.twin_center_x, (t - start) / 0.5);
        } else if t > end {
            let q = (t - end) / 0.6;
            scale = 1.0 - 0.6 * q.min(1.0);
            twin_x = lerp(self.twin_center_x, cx.pet_center_x(), q);
        }
        if t < end + 0.6 {
            look.twin = Some(Twin {
                frame: p.frame(Eyes::Happy, Pose::Stand(0)),
                center_x: twin_x,
                scale,
                y_offset: (-hop).max(0.0) * 3.0,
                facing_left: self.twin_center_x > cx.pet_center_x(),
            });
        }
        if t > start + 0.9 && t < end - 0.2 {
            // a ball passed back and forth
            let f = ((t - start - 0.9) / 1.2) % 2.0;
            let q = if f < 1.0 { f } else { 2.0 - f };
            let x = lerp(cx.pet_center_x(), self.twin_center_x, q) - 4.0;
            look.free
                .push(free(rows(BALL), x, 30.0 + 26.0 * (PI * q).sin(), PIXEL));
        }
        look
    }
}
