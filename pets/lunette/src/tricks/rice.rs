//! Pound Rice Cakes.

use crate::*;

pub struct Rice;

impl Trick for Rice {
    fn id(&self) -> &'static str {
        "rice"
    }
    fn title(&self) -> &'static str {
        "Pound Rice Cakes"
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 5.0 && cx.once(1) {
            cx.say("ta-da!", 1.8);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let eyes = cx.blink_eyes();
        let p = &cx.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        // Pounds rice in the mortar, then proudly holds up a rice cake.
        look.frame = p.frame(if t < 5.0 { eyes } else { Eyes::Happy }, Pose::Stand(0));
        look.props = vec![prop(rows(LUNETTE_MORTAR), 14.0, 0.0)];
        if t < 5.0 {
            look.props.push(prop(rows(LUNETTE_MALLET), 14.0, 3.0 + ((t * 2.0) as i64 % 2) as f64 * 4.0));
        } else {
            look.props.push(prop(rows(LUNETTE_CAKE), 14.0, 5.0));
            if beat > 0.0 {
                look.free = vec![cx.placed(HERO_SPARK, 21.0, 10.0)];
            }
        }
        look
    }
}
