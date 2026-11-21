//! Helpers shared by its tricks.

use crate::*;

/// Where Wisp-a-ling's glowing thread reaches: the pointer when it is near, otherwise a spot up ahead.
pub fn thread_target(cx: &Stage, t: f64) -> Pt {
    let edge = 8.0;
    let mut p = pt(
        cx.center_x(24.0) + 5.0 * PIXEL * t.sin(),
        (24.0 + 5.0 * t.cos()) * PIXEL,
    );
    if cx.mouse.y < cx.height + 160.0 && (cx.mouse.x - cx.pet_center_x()).abs() < 320.0 {
        p = cx.mouse;
    }
    pt(
        p.x.max(edge).min(cx.width - edge),
        p.y.max(edge).min(cx.height - edge),
    )
}
