//! Sprout a Flower.

use crate::*;

pub struct Bloom;

impl Trick for Bloom {
    fn id(&self) -> &'static str {
        "bloom"
    }
    fn title(&self) -> &'static str {
        "Sprout a Flower"
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let p = &cx.pet;
        let mut look = cx.look();
        // A flower pushes up through the moss while he sleeps, then folds away again.
        look.frame = p.frame(
            if t > 3.0 { Eyes::Happy } else { Eyes::Closed },
            Pose::Stand(0),
        );
        if t > 1.0 && t < 7.0 {
            look.props = vec![prop(
                rows(HERO_FLOWER),
                7.0,
                (14.0 + (t - 1.0).min(3.0)).round(),
            )];
        }
        look
    }
}
