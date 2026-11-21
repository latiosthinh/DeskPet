//! What it does while you type.

use crate::*;

pub struct Polish;

impl Typing for Polish {
    fn look(&self, cx: &Stage, _t: f64) -> Look {
        let mut look = cx.look();
        // Ronin polishes his katana, the cloth sliding up and down the blade.
        let slide = (2.5 * ((cx.now - cx.mode_start) * 5.0).sin()).round();
        look.props.push(prop(rows(SAMURAI_KATANA), 12.0, 4.0));
        look.props.push(prop(rows(SAMURAI_CLOTH), 18.0 + slide, 4.0));
        look
    }
}
