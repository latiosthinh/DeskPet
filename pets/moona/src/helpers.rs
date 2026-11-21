//! Helpers shared by its tricks.

use crate::*;

pub const MOON_DOWN: f64 = 3.8;
pub const MOONBEAMS: usize = 11;

/// She raises her glaive to the moon, and lowers it; how many beams strike the ground.
pub const MOON_UP: f64 = 0.3;

pub const POUNCE_LAND: f64 = 1.25;

/// When Moona leaves the ground in her pounce, and when she lands.
pub const POUNCE_UP: f64 = 0.3;

pub const THROW_BACK: f64 = 1.75;

/// Moona's glaive leaves her hand, and is back.
pub const THROW_OUT: f64 = 0.22;

/// Triplicate: the copies appear, the glaives start whirling, are caught, and the copies go.
pub const TRI_APPEAR: f64 = 0.45;

pub const TRI_CATCH: f64 = 4.0;

pub const TRI_SPIN: f64 = 0.8;

pub const TRI_VANISH: f64 = 4.55;

/// Where Moona's two copies stand when she triplicates: either side of her, inside the pen.
pub fn copy_centers(cx: &Stage) -> [f64; 2] {
    let c = cx.pet_center_x();
    let w = cx.pet.size().0;
    let (lo, hi) = (w * 0.45, (cx.width - w * 0.45).max(w * 0.45));
    [(c - w * 0.8).clamp(lo, hi), (c + w * 0.8).clamp(lo, hi)]
}

/// Where Moona's thrown glaive is (pen points, its centre), `q` of the way round its loop: out
/// ahead of her, high on the way out and lower on the way back, then into her hand.
pub fn glaive_at(cx: &Stage, q: f64) -> (f64, f64) {
    let (anchor, px) = cx
        .pet
        .picture
        .as_ref()
        .map_or((0.0, 1.0), |p| (p.anchors[0], p.px));
    let dir = if cx.facing_left { -1.0 } else { 1.0 };
    let hand_x = cx.pet_center_x() + dir * (MOONA_HAND.0 - anchor) * px;
    let hand_y = MOONA_HAND.1 * px;
    let room = if cx.facing_left {
        hand_x - 14.0
    } else {
        cx.width - 14.0 - hand_x
    };
    let rx = (room / 2.0).clamp(18.0, 70.0);
    let th = q.clamp(0.0, 1.0) * 2.0 * PI;
    let rise = if th < PI { 30.0 } else { 12.0 };
    (
        hand_x + dir * rx * (1.0 - th.cos()),
        hand_y + rise * th.sin(),
    )
}

pub fn glaive_copy(cx: &Stage, &(x, y, angle, faint): &(f64, f64, f64, bool)) -> FreeSprite {
    glaive_sprite(cx, x, y, angle, faint)
}

/// The spinning glaive centred at (x, y), turned `angle` radians; a faint one for its trail.
pub fn glaive_sprite(cx: &Stage, x: f64, y: f64, angle: f64, faint: bool) -> FreeSprite {
    let px = cx.pet.picture.as_ref().map_or(PIXEL / 2.0, |p| p.px);
    let grid = crescent(angle, 7.0, faint);
    let half = grid.len() as f64 * px / 2.0;
    FreeSprite {
        own_colors: true,
        ..free(grid, x - half, y - half, px)
    }
}

/// A shaft of moonlight `width` cells wide centred on x, reaching `reach` of the way down from
/// the top of the pen; faint as it fades.
pub fn moonbeam(cx: &Stage, x: f64, width: usize, reach: f64, faint: bool) -> FreeSprite {
    let px = PIXEL / 2.0;
    let rows_n = ((cx.height / px).ceil() * reach).round().max(1.0) as usize;
    // Brightest down the middle, softer to the sides, and fading out toward the top.
    let levels = ['.', 'R', 'S', 'T', 'U'];
    let mid = (width as f64 - 1.0) / 2.0;
    let across: Vec<usize> = (0..width)
        .map(|c| {
            let d = (c as f64 - mid).abs() / (width as f64 / 2.0);
            let level = if d > 0.75 {
                1
            } else if d > 0.5 {
                2
            } else if d > 0.2 {
                3
            } else {
                4
            };
            if faint {
                level - 1
            } else {
                level
            }
        })
        .collect();
    let grid: Rows = (0..rows_n)
        .map(|r| {
            let dim = if r * 10 < rows_n * 3 { 1 } else { 0 };
            across
                .iter()
                .map(|&l| levels[l.saturating_sub(dim)])
                .collect()
        })
        .collect();
    let left = x - width as f64 * px / 2.0;
    FreeSprite {
        own_colors: true,
        ..free(grid, left, cx.height - rows_n as f64 * px, px)
    }
}

/// Where the i-th narrow moonbeam strikes: spread over the pen, clear of Moona herself.
pub fn moonbeam_x(cx: &Stage, i: usize) -> f64 {
    let (lo, hi) = (10.0, (cx.width - 10.0).max(11.0));
    let f = (0.37 + i as f64 * 0.618_034) % 1.0;
    let mut x = lo + f * (hi - lo);
    let c = cx.pet_center_x();
    if (x - c).abs() < 48.0 {
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        x = c + side * (52.0 + 9.0 * (i % 3) as f64);
        if x < lo || x > hi {
            x = c - side * (52.0 + 9.0 * (i % 3) as f64);
        }
    }
    x.clamp(lo, hi)
}

/// One of her picture frames drawn as a separate figure centred on x (a copy of her).
pub fn picture_copy(cx: &Stage, frame: usize, center_x: f64) -> FreeSprite {
    let Some(pic) = cx.pet.picture.as_ref() else {
        return free(vec![], 0.0, 0.0, PIXEL);
    };
    let grid = &pic.frames[frame];
    let w = grid[0].len() as f64;
    let anchor = if cx.facing_left {
        w - pic.anchors[frame]
    } else {
        pic.anchors[frame]
    };
    let rows: Rows = grid
        .iter()
        .map(|r| {
            let s: String = r.iter().map(|&b| b as char).collect();
            if cx.facing_left {
                s.chars().rev().collect()
            } else {
                s
            }
        })
        .collect();
    FreeSprite {
        own_colors: true,
        ..free(rows, center_x - anchor * pic.px, 0.0, pic.px)
    }
}

/// A glaive whirling round a figure centred on x, on a flat ring at chest height: its trail and
/// itself, split into the parts behind the figure and in front. `phase` keeps the three apart.
pub fn whirl(
    cx: &Stage,
    x: f64,
    t: f64,
    phase: f64,
) -> (Vec<(f64, f64, f64, bool)>, Vec<(f64, f64, f64, bool)>) {
    let rx = cx.pet.size().0 * 0.55;
    let (mut behind, mut front) = (vec![], vec![]);
    for (lag, faint) in [(0.06, true), (0.03, true), (0.0, false)] {
        let tt = t - lag - TRI_SPIN;
        let a = phase * 2.1 + tt * 7.0 + 2.0 * tt * tt.min(1.0); // speeds up as it gets going
        let g = (
            x + rx * a.cos(),
            50.0 + 12.0 * a.sin(),
            (t - lag) * 17.0,
            faint,
        );
        if a.sin() > 0.0 {
            behind.push(g)
        } else {
            front.push(g)
        }
    }
    (behind, front)
}
