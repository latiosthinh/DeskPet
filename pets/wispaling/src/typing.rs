//! What it does while you type.

use crate::*;

pub struct Spool;

impl Typing for Spool {
    fn look(&self, cx: &Stage, t: f64) -> Look {
        let mut look = cx.look();
        // Wisp-a-ling winds glowing keystrokes onto a spool.
        look.props.push(prop(rows(HERO_THREAD_SPOOL), 15.0, 0.0));
        for i in 0..6 {
            let lift = 3.0 - (2.0 * (t * 7.0 + i as f64).sin()).round();
            look.props.push(prop(rows(HERO_SPARK), 12.0 + i as f64, lift));
        }
        look
    }
}
