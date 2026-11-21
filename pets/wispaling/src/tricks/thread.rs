//! Reach for the Pointer.

use crate::*;

pub struct Thread;

impl Trick for Thread {
    fn id(&self) -> &'static str {
        "thread"
    }
    fn title(&self) -> &'static str {
        "Reach for the Pointer"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        let target = thread_target(cx, t);
        if (target.x - cx.pet_center_x()).abs() > 6.0 {
            cx.facing_left = target.x < cx.pet_center_x();
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let d = cx.duration();
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // A glowing thread stretches out to the pointer while the wisp's glow pulses.
        look.frame = p.frame(eyes, Pose::Stand(0));
        look.palette.set('w', Some(if beat > 0.0 { hex(0xfff7d8) } else { hex(0xe9b878) }));
        let reach = if t < 2.0 {
            t / 2.0
        } else if t > d - 2.0 {
            (d - t).max(0.0) / 2.0
        } else {
            1.0
        };
        let target = thread_target(cx, t);
        let from = pt(cx.center_x(14.0), 5.0 * PIXEL);
        let end = pt(from.x + (target.x - from.x) * reach, from.y + (target.y - from.y) * reach);
        let steps = 2.max(((end.x - from.x).hypot(end.y - from.y) / 5.0) as usize);
        look.free = (0..=steps)
            .map(|i| {
                let q = i as f64 / steps as f64;
                free(rows(HERO_SPARK), from.x + (end.x - from.x) * q - 3.0, from.y + (end.y - from.y) * q - 3.0, 2.0)
            })
            .collect();
        look.free.push(free(rows(HERO_STAR), end.x - 5.0, end.y - 5.0, 2.0));
        look
    }
}
