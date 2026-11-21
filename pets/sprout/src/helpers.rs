//! Helpers shared by its tricks.

use crate::*;

pub fn butterfly_point(cx: &Stage, t: f64, d: f64) -> Pt {
    let lo: f64 = 16.0;
    let hi = (lo + 1.0).max(cx.width - 16.0);
    let (mid, half) = ((lo + hi) / 2.0, ((hi - lo) / 2.0 - 10.0).max(0.0));
    let x = mid + half * (t * 0.8).sin() + 8.0 * (t * 3.1).sin();
    let mut y = 46.0 + 14.0 * (t * 2.2).sin();
    if t > d - 1.5 {
        y += (t - (d - 1.5)) * 70.0; // flies away at the end
    }
    pt(x, y)
}
