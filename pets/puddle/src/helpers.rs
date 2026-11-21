//! Helpers shared by its tricks.

use crate::*;

pub fn glasses_sprite(cx: &Stage) -> FreeSprite {
    let half = PIXEL / 2.0;
    let start = 10.0;
    let width = GLASSES[0].len() as f64;
    let grid = if cx.facing_left { GLASSES.iter().map(|r| r.chars().rev().collect()).collect() } else { rows(GLASSES) };
    let x = cx.pet_x + (if cx.facing_left { 32.0 - start - width } else { start }) * half;
    free(grid, x, 11.0 * half, half)
}
