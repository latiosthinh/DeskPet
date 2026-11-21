//! Helpers shared by its tricks.

use crate::*;

pub const BLAST_HIT: f64 = 1.8;

pub const BLAST_SHOOT: f64 = 0.9;

pub const GROUND_EMBERS: &[&str] = &["8.8.8"];

/// Fires the meteor leaves on the ground: two flickers of a flame (6 yellow, 7 orange, 8 red), a low
/// one as it dies down, then embers.
pub const GROUND_FIRE_A: &[&str] = &["..7...", ".767..", ".7677.", "76667.", "866687", ".8888."];

pub const GROUND_FIRE_B: &[&str] = &["...7..", "..767.", ".7767.", ".76667", "786668", ".8888."];

pub const GROUND_FIRE_LOW: &[&str] = &["..7..", ".767.", "78887"];

pub const METEOR_END: f64 = 3.6;

pub const METEOR_FIRES: usize = 9;

pub const METEOR_LAND: f64 = 1.3;

/// How far the meteor rolls after it lands (points), and how many fires it leaves on the way.
pub const METEOR_ROLL: f64 = 90.0;

pub const METEOR_START: f64 = 0.5;

pub const ORB_LAND: f64 = 1.2;

/// Orbis: an orb leaves his hand and lands; the whirl starts and flashes; he fades out, moves
/// while he's gone, and fades back in.
pub const ORB_LAUNCH: f64 = 0.35;

pub const ORB_WHIRL_FLASH: f64 = 2.2;

pub const ORB_WHIRL_START: f64 = 0.3;

pub const SPIRITS_COME: f64 = 0.7;

pub const SPIRITS_GO: f64 = 7.3;

/// Orbis's spells: when the sunlight is called and strikes; when the tornado dies away; when the
/// meteor appears, lands and stops rolling; when the moon slash flies and breaks up; when his spirits come and go.
pub const SUN_CALL: f64 = 0.6;

pub const SUN_END: f64 = 2.1;

pub const SUN_HIT: f64 = 1.0;

pub const TORNADO_END: f64 = 3.3;

pub const VANISH_BACK: f64 = 2.3;

pub const VANISH_GONE: f64 = 1.3;

pub const VANISH_MOVE: f64 = 1.7;

/// A burning rock: falling, its flames stream out behind it (up and back); rolling on the ground,
/// they flicker above it. Drawn for one heading right.
pub fn meteor(t: f64, falling: bool) -> Rows {
    let n = 26;
    let (cx, cy) = (18.0, 19.0); // the rock, near the bottom right
    (0..n)
        .map(|r| {
            (0..n)
                .map(|c| {
                    let (dx, dy) = (c as f64 - cx, r as f64 - cy);
                    let d = (dx * dx + dy * dy).sqrt();
                    if d <= 4.2 {
                        return if (c + r) % 3 == 0 { '8' } else { '9' };
                    }
                    if d <= 5.4 {
                        return '7';
                    }
                    // The flames: along the way back (up-left when falling, straight up when rolling).
                    let (ux, uy) = if falling {
                        (-0.707, -0.707)
                    } else {
                        (0.0, -1.0)
                    };
                    let u = dx * ux + dy * uy;
                    let v = (dx * uy - dy * ux).abs();
                    let len = if falling { 18.0 } else { 9.0 };
                    if u <= 0.0 || u >= len {
                        return '.';
                    }
                    let width = 5.4 * (1.0 - u / len) + 0.8 * (u * 0.9 + t * 25.0).sin();
                    if v > width {
                        '.'
                    } else if v < width * 0.35 && u < len * 0.6 {
                        '6'
                    } else if u < len * 0.75 {
                        '7'
                    } else {
                        '8'
                    }
                })
                .collect()
        })
        .collect()
}

/// Rows flipped left to right when `flip`.
pub fn mirrored(grid: Rows, flip: bool) -> Rows {
    if flip {
        grid.iter().map(|r| r.chars().rev().collect()).collect()
    } else {
        grid
    }
}

/// A crescent of moonlight standing on end, bulging the way it flies (right; mirror it for left):
/// what is inside one circle and outside another set back from it. `size` 1 is about 32 cells
/// tall; a faint one is all halo, for its trail.
pub fn moon_slash(size: f64, faint: bool) -> Rows {
    let r = 17.0 * size;
    let (inner, back) = (r * 0.95, r * 0.35);
    let x0 = r * 0.35 + 1.0; // the circle's centre column: the tips reach a little behind it
    let (w, h) = (
        (x0 + r + 3.0).ceil() as usize,
        (2.0 * r + 4.0).ceil() as usize,
    );
    let my = (h as f64 - 1.0) / 2.0;
    (0..h)
        .map(|y| {
            (0..w)
                .map(|x| {
                    let (dx, dy) = (x as f64 - x0, y as f64 - my);
                    let outer_d = (dx * dx + dy * dy).sqrt();
                    let inner_d = ((dx + back).powi(2) + dy * dy).sqrt();
                    if outer_d <= r && inner_d > inner {
                        if faint {
                            '+'
                        } else if r - outer_d < 1.2 || inner_d - inner < 1.2 {
                            '#'
                        } else {
                            '0'
                        }
                    } else if dx > 0.0 && outer_d <= r + 1.8 && inner_d > inner - 1.0 {
                        '+'
                    } else {
                        '.'
                    }
                })
                .collect()
        })
        .collect()
}

/// A grid drawn in the pet's own colours, centred on x with its bottom `bottom` points up.
pub fn spell_sprite(grid: Rows, x: f64, bottom: f64, px: f64, _mirrored: bool) -> FreeSprite {
    let w = grid.first().map_or(0, |r| r.chars().count()) as f64 * px;
    FreeSprite {
        own_colors: true,
        ..free(grid, x - w / 2.0, bottom, px)
    }
}

/// A whirlwind `h` cells tall: a funnel wide at the top and narrow at the ground, its stripes
/// turning; thin and faint as it dies away.
pub fn tornado(t: f64, h: usize, dying: bool) -> Rows {
    let total = 40;
    (0..h)
        .map(|r| {
            let rf = r as f64;
            let down = rf / h as f64; // 0 at the top, 1 at the ground
            let w = 2.5 + 15.0 * (1.0 - down);
            let off = 4.0 * (rf * 0.22 + t * 9.0).sin() * (1.0 - down);
            (0..total)
                .map(|c| {
                    let x = c as f64 - total as f64 / 2.0 - off;
                    if x.abs() > w {
                        '.'
                    } else if dying || x.abs() > w - 1.2 {
                        '5'
                    } else if ((x + t * 30.0 + rf * 0.8) / 2.5).floor().rem_euclid(2.0) == 0.0 {
                        '4'
                    } else {
                        '5'
                    }
                })
                .collect()
        })
        .collect()
}

/// A shaft of light `width` cells wide centred on x, reaching `reach` of the way down from the
/// top of the pen, brightest down the middle: `levels` from none to brightest.
pub fn light_shaft(
    cx: &Stage,
    x: f64,
    width: usize,
    reach: f64,
    levels: [char; 5],
    faint: bool,
) -> FreeSprite {
    let px = PIXEL / 2.0;
    let rows_n = ((cx.height / px).ceil() * reach).round().max(1.0) as usize;
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
    FreeSprite {
        own_colors: true,
        ..free(
            grid,
            x - width as f64 * px / 2.0,
            cx.height - rows_n as f64 * px,
            px,
        )
    }
}

/// One of his orbs (0 orange, 1 blue, 2 pink) centred at (x, y); a see-through one for its trail.
pub fn orb_sprite(cx: &Stage, kind: usize, x: f64, y: f64, faint: bool) -> FreeSprite {
    let px = cx.pet.picture.as_ref().map_or(PIXEL / 2.0, |p| p.px);
    let trail = ['S', 'T', 'U'][kind % 3];
    let grid: Rows = WITCHER_ORBS[kind % 3]
        .iter()
        .map(|r| {
            r.chars()
                .map(|ch| if faint && ch != '.' { trail } else { ch })
                .collect()
        })
        .collect();
    let (w, h) = (grid[0].len() as f64 * px, grid.len() as f64 * px);
    FreeSprite {
        own_colors: true,
        ..free(grid, x - w / 2.0, y - h / 2.0, px)
    }
}

/// Where his moon slash is at time `t` of the trick: from his hand straight out to the edge of the pen.
pub fn slash_x(cx: &Stage, t: f64) -> f64 {
    let (hand_x, _) = witcher_hand(cx, 0.0);
    let q = ((t - BLAST_SHOOT) / (BLAST_HIT - BLAST_SHOOT)).clamp(0.0, 1.0);
    hand_x + (spell_x(cx, 400.0) - hand_x) * q
}

/// Where a spell lands: `ahead` points in front of him, kept inside the pen.
pub fn spell_x(cx: &Stage, ahead: f64) -> f64 {
    let dir = if cx.facing_left { -1.0 } else { 1.0 };
    (cx.pet_center_x() + dir * ahead).clamp(12.0, (cx.width - 12.0).max(13.0))
}

/// Where his two fire spirits stand: either side of him, inside the pen, wandering a little.
pub fn spirit_xs(cx: &Stage, t: f64) -> [f64; 2] {
    let c = cx.pet_center_x();
    let w = cx.pet.size().0;
    let (lo, hi) = (12.0, (cx.width - 12.0).max(13.0));
    [
        (c - w * 0.62 + 8.0 * (t * 1.3).sin()).clamp(lo, hi),
        (c + w * 0.62 + 8.0 * (t * 1.1 + 2.0).sin()).clamp(lo, hi),
    ]
}

/// Where a cast orb is (pen points, its centre), `q` of the way along its flight: from his hand,
/// straight out ahead, dipping a little as it goes.
pub fn witcher_hand(cx: &Stage, q: f64) -> (f64, f64) {
    let px = cx.pet.picture.as_ref().map_or(1.0, |p| p.px);
    let dir = if cx.facing_left { -1.0 } else { 1.0 };
    let hand_x = cx.pet_center_x() + dir * WITCHER_HAND.0 * px;
    let hand_y = WITCHER_HAND.1 * px;
    let room = if cx.facing_left {
        hand_x - 14.0
    } else {
        cx.width - 14.0 - hand_x
    };
    let reach = room.clamp(30.0, 150.0);
    let q = q.clamp(0.0, 1.0);
    (hand_x + dir * reach * q, hand_y - 22.0 * q * q)
}

/// His three orbs going round on a wheel behind his shoulder (faster while you type), each with
/// a faint trail.
pub fn orb_wheel(cx: &Stage) -> Vec<FreeSprite> {
    let dir = if cx.facing_left { -1.0 } else { 1.0 };
    let (x, y, r) = (cx.pet_center_x() - dir * 30.0, 70.0, 19.0);
    let speed = if cx.mode == Mode::Typing { 4.0 } else { 1.4 };
    let mut out = vec![];
    for (lag, faint) in [(0.14, true), (0.0, false)] {
        for k in 0..3 {
            let a = -dir * (cx.clock - lag) * speed + k as f64 * 2.0 * PI / 3.0;
            out.push(orb_sprite(cx, k, x + r * a.cos(), y + r * a.sin(), faint));
        }
    }
    out
}
