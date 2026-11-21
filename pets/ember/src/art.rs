//! Ember the fox's own pixel art.

use crate::*;

pub const EGG_DOWN: &[&str] = &[".iii.", "iiiii"];

pub const EGG_UP: &[&str] = &[".iyi.", "iiiii"];

pub const FIRE_A: &[&str] = &["...y...", "..yoy..", ".yooy..", ".yoooy.", "nknnnkn"];

pub const FIRE_B: &[&str] = &["....y..", "..yoy..", "..yooy.", ".yoooy.", "nknnnkn"];

pub const FLASHLIGHT: &[&str] = &["kkkkk", "kgggy", "kkkkk"];

pub const PAN: &[&str] = &["....kkkkkk", "nnnnkddddk"];

pub const ROCKET: &[&str] = &[".k.", "krk", "krk", "kyk", ".n."];

pub fn stick(toasted: bool) -> Rows {
    if toasted {
        rows(&["....yy", "...nyy", "..n...", ".n....", "n....."])
    } else {
        rows(&["....ii", "...nii", "..n...", ".n....", "n....."])
    }
}
