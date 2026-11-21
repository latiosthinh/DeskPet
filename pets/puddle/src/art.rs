//! Puddle the slime's own pixel art.

use crate::*;

pub const BALL: &[&str] = &["rr", "rr"];

pub const FOOD: [&[&str]; 3] = [
    &["..G.", ".rr.", "rrrr", "rrrr", ".rr."], // apple
    &[".yy.", "ynyy", "yyny", ".yy."],         // cookie
    &["..i..", ".iii.", "iiiii", "ikkki"],     // rice ball
];

/// Red-framed glasses on a half-size grid (2 pt cells), so each thin rim fits around an eye
/// (eyes at sprite columns 9 and 12). Starts at half-column 10; the arm runs back toward the ear.
pub const GLASSES: &[&str] = &[".......rrrr..rrrr", "rrrrrrrr..rrrr..r", ".......r..r..r..r", ".......r..r..r..r", ".......r..r..r..r", ".......rrrr..rrrr"];

pub fn ice_cream(left: f64) -> Rows {
    let scoop: [&str; 3] = if left > 0.66 {
        ["..PP.", ".PPPP", "PPPPP"]
    } else if left > 0.33 {
        [".....", ".PPP.", "PPPPP"]
    } else if left > 0.05 {
        [".....", ".....", ".PPP."]
    } else {
        [".....", ".....", "....."]
    };
    rows(&[scoop[0], scoop[1], scoop[2], "nyyyn", ".nyn.", "..n.."])
}
