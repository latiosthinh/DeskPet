//! Helpers shared by its tricks.

use crate::*;

pub fn yarn_x(cx: &Stage, t: f64) -> f64 {
    let lo: f64 = 20.0;
    let hi = (lo + 1.0).max(cx.width - 20.0);
    let (mid, half) = ((lo + hi) / 2.0, ((hi - lo) / 2.0 - 6.0).max(0.0));
    mid + half * (t * 0.9 + 0.6).sin()
}
