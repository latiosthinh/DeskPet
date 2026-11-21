//! Shared pixel art for tricks and pastimes, drawn in prop colours (`prop_palette`).
//! Anything only one pet uses lives in that pet's own crate.

use crate::art::Rows;

pub const HERO_FLOWER: &[&str] = &["..P..", ".PyP.", "..P..", "..G..", ".GG.."];

pub const HERO_PAD: &[&str] = &["kkkkkkkk", "kiiiiiik", "kikikikk", "kiiiiiik", "kkkkkkkk"];

pub const HERO_PUFF: &[&str] = &["..WW.W..", ".WWWWWW.", "WWWWWWWW", ".WWWWWW.", "..W.WW.."];

pub const HERO_SPARK: &[&str] = &[".C.", "CCC", ".C."];

pub const HERO_STAR: &[&str] = &["..y..", ".yyy.", "yyyyy", ".yyy.", "..y.."];

pub const HERO_Z: &[&str] = &["kkkk", "..k.", ".k..", "kkkk"];

pub const TWINKLE: &[&str] = &[".B.", "BWB", ".B."];

pub const PASTIME_BALL_C: &[&str] = &["cc", "cc"];

pub const PASTIME_BALL_R: &[&str] = &["rr", "rr"];

pub const PASTIME_BALL_Y: &[&str] = &["yy", "yy"];

pub const PASTIME_BUBBLE: &[&str] = &[".c.", "c.c", ".c."];

pub const PASTIME_BUTTERFLY_A: &[&str] = &["P.P", ".k.", "P.P"];

pub const PASTIME_BUTTERFLY_B: &[&str] = &["...", "PkP", "..."];

pub const PASTIME_NEEDLES_A: &[&str] = &["n...n", ".n.n.", "..n.."];

pub const PASTIME_NEEDLES_B: &[&str] = &[".n.n.", ".n.n.", "..n.."];

pub const PASTIME_WAND: &[&str] = &[".ii.", "i..i", ".ii.", ".n..", "n..."];

pub const PASTIME_YOYO: &[&str] = &[".r.", "rkr", ".r."];

pub fn pool(width: usize) -> Rows {
    let w = width.max(2);
    vec![
        format!(".{}.", "C".repeat(w - 2)),
        "c".repeat(w),
        "c".repeat(w),
    ]
}

/// A crescent blade of radius `r` cells, its hollow facing `angle` radians (counter-clockwise from
/// the right): a bright core inside a glowing edge, with a soft halo. Faint: all trail colour.
pub fn crescent(angle: f64, r: f64, faint: bool) -> Rows {
    let n = (2.0 * r).ceil() as usize + 3;
    let c = (n as f64 - 1.0) / 2.0;
    let (s, co) = angle.sin_cos();
    let off = (0.6 * r * co, 0.6 * r * s);
    let inner = 0.82 * r;
    (0..n)
        .map(|row| {
            (0..n)
                .map(|col| {
                    let (x, y) = (col as f64 - c, c - row as f64);
                    let d1 = r - (x * x + y * y).sqrt();
                    let d2 = ((x - off.0).powi(2) + (y - off.1).powi(2)).sqrt() - inner;
                    let edge = d1.min(d2);
                    if edge >= 0.0 {
                        if faint {
                            'P'
                        } else if edge >= 1.0 {
                            'N'
                        } else {
                            'G'
                        }
                    } else if edge > -1.0 && !faint {
                        'O'
                    } else {
                        '.'
                    }
                })
                .collect()
        })
        .collect()
}
