//! Timings shared by Ronin's tricks, in seconds from the start of the trick, and his bow.

use crate::*;

pub const BAMBOO_CUT: f64 = 2.0;

pub const DRAW_CUT: f64 = 2.6;

pub const FLY_CATCH: f64 = 3.6;

pub const FLY_FREE: f64 = 6.2;

pub const SPIN: (f64, f64) = (1.2, 3.4);

pub const TEA_SIP: (f64, f64) = (3.0, 4.6);

/// A bow: the head and shoulders (top nine rows) tip `cols` forward and one row down.
pub fn lean(mut f: Frame, cols: usize) -> Frame {
    for r in (1..=9).rev() {
        let mut out = [b'.'; 16];
        out[cols..].copy_from_slice(&f[r - 1][..16 - cols]);
        f[r] = out;
    }
    f[0] = [b'.'; 16];
    f
}
