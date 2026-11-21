//! Turn into a Frog / Turn into a Sheep: one trick, played with two different critters.

use crate::*;

/// Waves the wand, vanishes in a puff, bounces about as a critter, then puffs back.
pub struct TurnInto {
    pub id: &'static str,
    pub title: &'static str,
    /// What the critter says once it appears.
    pub says: &'static str,
    pub critter: &'static [&'static str],
}

pub const FROG: TurnInto = TurnInto { id: "frog", title: "Turn into a Frog", says: "ribbit?!", critter: HERO_FROG };
pub const SHEEP: TurnInto = TurnInto { id: "sheep", title: "Turn into a Sheep", says: "baa…", critter: HERO_SHEEP };

impl Trick for TurnInto {
    fn id(&self) -> &'static str {
        self.id
    }
    fn title(&self) -> &'static str {
        self.title
    }

    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if t > 3.0 && cx.once(1) {
            cx.say(self.says, 1.5);
        }
    }

    fn look(&self, cx: &Stage, t: f64) -> Look {
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s
        let mut look = cx.look();
        look.frame = cx.pet.frame(if t >= 7.0 { Eyes::Happy } else { cx.blink_eyes() }, Pose::Stand(0));
        let hidden = [[b'.'; 16]; 16];
        if t < 2.0 {
            look.props = vec![prop(rows(HERO_WAND), 14.0, 4.0 + beat)];
        } else if t < 3.0 || (6.0..7.0).contains(&t) {
            look.frame = hidden;
            look.free = vec![cx.placed(HERO_PUFF, 4.0, 5.0)];
        } else if t < 6.0 {
            look.frame = hidden;
            look.free = vec![cx.placed(self.critter, 4.0, (3.0 * (t * 5.0).sin().abs()).round())];
        }
        look
    }
}
