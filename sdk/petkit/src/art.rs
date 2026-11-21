//! Colours, palettes, frames and picture pets: the building blocks of pet art.
//! Grids are rows of characters, top row first, facing right; each character is looked up in a palette.

pub const PIXEL: f64 = 4.0; // points per art pixel
pub const PET_SIZE: f64 = 16.0 * PIXEL; // sprite is 16×16 art pixels
pub const PROP_ROOM: f64 = 36.0; // space kept inside the pen for props held in front

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
    pub fn with_alpha(self, a: f64) -> Color {
        Color {
            a: (a.clamp(0.0, 1.0) * 255.0).round() as u8,
            ..self
        }
    }
}

pub fn rgb(r: f64, g: f64, b: f64) -> Color {
    let c = |v: f64| (v * 255.0).round() as u8;
    Color::new(c(r), c(g), c(b), 255)
}

/// Colours for grid characters (ASCII only).
#[derive(Clone, Copy)]
pub struct Palette(pub(crate) [Option<Color>; 128]);

impl Palette {
    pub fn new(entries: &[(char, Color)]) -> Palette {
        let mut p = Palette([None; 128]);
        for &(k, c) in entries {
            p.set(k, Some(c));
        }
        p
    }
    pub fn get(&self, ch: u8) -> Option<Color> {
        self.0.get(ch as usize).copied().flatten()
    }
    pub fn set(&mut self, ch: char, c: Option<Color>) {
        self.0[ch as usize] = c;
    }
    /// Every colour see-through by `a` (0 gone, 1 as it was).
    pub fn faded(&self, a: f64) -> Palette {
        let mut out = *self;
        for c in out.0.iter_mut().flatten() {
            *c = c.with_alpha(c.a as f64 / 255.0 * a);
        }
        out
    }
}

/// Colours for props. "w" (the paw on the laptop) comes from the pet's own body colour.
pub fn prop_palette() -> Palette {
    Palette::new(&[
        ('k', rgb(0.20, 0.16, 0.24)), // outline
        ('g', rgb(0.66, 0.68, 0.74)), // laptop
        ('b', rgb(0.45, 0.78, 1.00)), // laptop screen
        ('B', rgb(0.85, 0.97, 1.00)), // laptop screen, bright line
        ('n', rgb(0.58, 0.38, 0.22)), // broom handle
        ('y', rgb(0.96, 0.78, 0.36)), // broom straw
        ('r', rgb(0.87, 0.34, 0.38)), // book cover
        ('s', rgb(0.85, 0.82, 0.78)), // turning page
        ('c', rgb(0.55, 0.82, 1.00)), // water
        ('o', rgb(0.93, 0.46, 0.32)), // bowl
        ('i', rgb(1.00, 1.00, 1.00)), // rice, book pages, egg white, marshmallow
        ('d', rgb(0.30, 0.30, 0.35)), // frying pan
        ('C', rgb(0.80, 0.94, 1.00)), // water surface
        ('P', rgb(1.00, 0.62, 0.76)), // petals, butterfly, ice cream
        ('G', rgb(0.33, 0.64, 0.30)), // stems
        ('N', rgb(0.40, 0.26, 0.15)), // dark soil
        ('W', rgb(0.95, 0.96, 0.99)), // cloud
        ('T', rgb(0.82, 0.64, 0.42)), // cardboard
        ('U', rgb(0.66, 0.49, 0.31)), // cardboard tape and flaps
        ('L', rgb(0.50, 0.80, 0.40)), // leaves, bamboo
    ])
}

/// Apple's light-mode system colours, which the Mac version uses for particles.
pub mod sys_color {
    use super::Color;
    pub const PINK: Color = Color::new(255, 45, 85, 255);
    pub const BROWN: Color = Color::new(162, 132, 94, 255);
    pub const YELLOW: Color = Color::new(255, 204, 0, 255);
    pub const ORANGE: Color = Color::new(255, 149, 0, 255);
    pub const TEAL: Color = Color::new(48, 176, 199, 255);
    pub const PURPLE: Color = Color::new(175, 82, 222, 255);
    pub const GREEN: Color = Color::new(52, 199, 89, 255);
    pub const BLUE: Color = Color::new(0, 122, 255, 255);
    pub const PLAIN_BROWN: Color = Color::new(153, 102, 51, 255); // NSColor.brown
    pub const SECONDARY_LABEL: Color = Color::new(110, 110, 118, 255);
    pub const SPARKLES: [Color; 6] = [YELLOW, PINK, ORANGE, TEAL, PURPLE, GREEN];
}

#[derive(Clone, Copy, PartialEq)]
pub enum Eyes {
    Open,
    Closed,
    Happy,
}

/// Standing on one of the two walking-feet rows, or sitting down.
#[derive(Clone, Copy, PartialEq)]
pub enum Pose {
    Stand(usize),
    Sit,
}

/// A 16×16 frame: a hero/critter pet, eyes and feet included, facing right, row 0 at the top.
pub type Frame = [[u8; 16]; 16];

/// A pet drawn from detailed pictures (any size) instead of the 16×16 grid, one per pose.
pub struct Picture {
    /// Indexed by `eyes * 3 + pose`: eyes open, closed, happy; poses standing (two walking
    /// steps) and sitting. Rows top to bottom; the bottom row stands on the ground.
    /// Frames past the first nine are a pet's own extra poses (Moona's leap, sleep, ...).
    pub frames: Vec<Vec<Vec<u8>>>,
    /// The column of each frame that stands at the pet's centre, so poses of different
    /// widths line up.
    pub anchors: Vec<f64>,
    pub width: usize, // of the standing picture
    pub height: usize,
    pub px: f64, // points per art pixel
    /// Stands still on the sitting frame when idle (its standing frames are the two walking steps).
    pub idle_sits: bool,
    /// How far (in 16-grid columns) held props move forward, to clear a long body.
    pub prop_shift: f64,
    /// A walk cycle of more than two steps (frame indices), when the picture has one.
    pub walk: Vec<usize>,
    /// Frames for sleeping and eating, when the picture has its own.
    pub sleep: Option<usize>,
    pub eat: Option<usize>,
}

/// Marker bytes start here; they are past the 128-entry palette, so they never draw.
const PICTURE_MARK: u8 = 200;

impl Picture {
    /// A picture from separately drawn poses: (rows, anchor column) each, standing first.
    /// The nine standard frames are built by the caller; `extra` are appended after them.
    pub fn from_frames(frames: Vec<(Vec<Vec<u8>>, f64)>, px: f64) -> Picture {
        let (width, height) = (frames[0].0[0].len(), frames[0].0.len());
        let anchors = frames.iter().map(|f| f.1).collect();
        let frames = frames.into_iter().map(|f| f.0).collect();
        Picture {
            frames,
            anchors,
            width,
            height,
            px,
            idle_sits: false,
            prop_shift: 0.0,
            walk: vec![],
            sleep: None,
            eat: None,
        }
    }

    /// A marker frame for any frame by index (the extra poses come after the first nine).
    pub fn marker_index(i: usize) -> Frame {
        let mut out = [[b'.'; 16]; 16];
        out[0][0] = PICTURE_MARK + i as u8;
        out
    }

    /// Which frame a marker frame asks for.
    pub fn index_for(&self, marker: &Frame) -> usize {
        let i = marker
            .iter()
            .flatten()
            .find(|&&b| b >= PICTURE_MARK)
            .map_or(0, |&b| (b - PICTURE_MARK) as usize);
        i.min(self.frames.len() - 1)
    }

    /// What `Pet::frame` returns for a picture pet: an empty 16×16 frame holding one marker byte
    /// that says which picture frame to draw. Trick code passes frames around without looking
    /// inside, so picture pets work with every activity and trick.
    pub fn marker(eyes: Eyes, pose: Pose) -> Frame {
        let e = match eyes {
            Eyes::Open => 0,
            Eyes::Closed => 1,
            Eyes::Happy => 2,
        };
        let p = match pose {
            Pose::Stand(step) => step % 2,
            Pose::Sit => 2,
        };
        let mut out = [[b'.'; 16]; 16];
        out[0][0] = PICTURE_MARK + (e * 3 + p) as u8;
        out
    }
}

/// A four-step walk cycle for a four-legged picture: each leg (a separate blob of pixels below row
/// `hip`) swings about the hip, its rows shifting sideways in proportion to their depth, and is
/// lifted `lift` rows while it passes forward. Diagonal legs move together.
pub fn gait(grid: &[Vec<u8>], hip: usize, swing: f64, lift: usize) -> [Vec<Vec<u8>>; 4] {
    let (h, w) = (grid.len(), grid[0].len());
    // Find the legs: 4-connected blobs of at least ten cells below the hip.
    let mut seen = vec![vec![false; w]; h];
    let mut legs: Vec<(f64, Vec<(usize, usize)>)> = Vec::new();
    for r in hip..h {
        for c in 0..w {
            if grid[r][c] == b'.' || seen[r][c] {
                continue;
            }
            seen[r][c] = true;
            let (mut stack, mut cells) = (vec![(r, c)], Vec::new());
            while let Some((y, x)) = stack.pop() {
                cells.push((y, x));
                let near = [
                    (y + 1, x),
                    (y.wrapping_sub(1), x),
                    (y, x + 1),
                    (y, x.wrapping_sub(1)),
                ];
                for (y2, x2) in near {
                    if (hip..h).contains(&y2) && x2 < w && grid[y2][x2] != b'.' && !seen[y2][x2] {
                        seen[y2][x2] = true;
                        stack.push((y2, x2));
                    }
                }
            }
            if cells.len() >= 10 {
                let cx = cells.iter().map(|p| p.1 as f64).sum::<f64>() / cells.len() as f64;
                legs.push((cx, cells));
            }
        }
    }
    legs.sort_by(|a, b| a.0.total_cmp(&b.0));
    // Back to front: hind (back), hind (under), front (near), front (far). Pairs 0+2 and 1+3;
    // the far ones are drawn first so the near ones overlap them.
    let pair = |i: usize| if i % 2 == 0 { 0 } else { 2 };
    let order: Vec<usize> = if legs.len() == 4 {
        vec![0, 3, 1, 2]
    } else {
        (0..legs.len()).collect()
    };
    let phase = [(1.0, 0), (0.0, 0), (-1.0, 0), (0.0, 1)]; // (swing, lifted) through the cycle
    std::array::from_fn(|step| {
        let mut out = grid.to_vec();
        for (_, cells) in &legs {
            for &(r, c) in cells {
                out[r][c] = b'.';
            }
        }
        for &i in &order {
            let (sw, up) = phase[(step + pair(i)) % 4];
            for &(r, c) in &legs[i].1 {
                let dx = (sw * swing * (r - hip) as f64 / (h - 1 - hip) as f64).round() as i64;
                let (nr, nc) = (r - up * lift, c as i64 + dx);
                if (0..w as i64).contains(&nc) {
                    out[nr][nc as usize] = grid[r][c];
                }
            }
        }
        out
    })
}

/// A colour written as 0xRRGGBB.
pub const fn hex(v: u32) -> Color {
    Color::new((v >> 16) as u8, (v >> 8) as u8, v as u8, 255)
}

/// Grid rows for props and tricks.
pub type Rows = Vec<String>;

pub fn rows(r: &[&str]) -> Rows {
    r.iter().map(|s| s.to_string()).collect()
}

/// Props sit just in front of the body (sprite column 14 is the body's front outline).
pub mod props {
    use super::{rows, Rows};

    pub fn laptop_closed() -> Rows {
        rows(&["kkkkkkkkk.", "kgggggggk.", "kkkkkkkkk."])
    }

    /// Side view: keyboard on the floor, lid tilted back with its glowing screen facing the pet.
    pub fn laptop_open(paw_up: bool, flicker: bool) -> Rows {
        rows(&[
            "........kk",
            ".......kbk",
            if flicker { ".......kBk" } else { ".......kbk" },
            "......kbk.",
            if paw_up { "kwk...kbk." } else { ".kwk..kBk." },
            "kkkkkkkkk.",
            "kgggggggk.",
            "kkkkkkkkk.",
        ])
    }

    pub fn broom(swish: bool) -> Rows {
        let mut out = rows(&["n......", ".n.....", "..n....", "...n..."]);
        for s in ["..kkk..", ".kyyyk.", "kyyyyyk"] {
            out.push(if swish {
                format!(".{}", &s[..s.len() - 1])
            } else {
                s.to_string()
            });
        }
        out
    }

    /// An open book held up in front.
    pub fn book(flip: bool) -> Rows {
        rows(&[
            ".kk.kk.",
            if flip { "kiiksik" } else { "kiikiik" },
            "kiikiik",
            "kiikiik",
            "krrrrrk",
        ])
    }

    /// `left` goes from 1 (full) to 0 (empty).
    pub fn cup(left: f64) -> Rows {
        let full = ((left * 3.0).ceil() as i32).clamp(0, 3);
        let mut out = rows(&["k...k"]);
        for i in 0..3 {
            out.push(if i >= 3 - full { "kccck" } else { "k...k" }.to_string());
        }
        out.push(".kkk.".to_string());
        out
    }

    pub fn bowl(left: f64) -> Rows {
        let rice: [&str; 2] = if left > 0.5 {
            ["..iiiii..", ".iiiiiii."]
        } else if left > 0.05 {
            [".........", ".iiiiiii."]
        } else {
            [".........", "........."]
        };
        rows(&[rice[0], rice[1], "koooooook", ".koooook."])
    }
}

/// Turns a 16×16 frame a quarter turn clockwise `turns` times.
pub fn rotated(frame: Frame, turns: usize) -> Frame {
    let mut g = frame;
    for _ in 0..(turns % 4) {
        let mut out = [[b'.'; 16]; 16];
        for (r, row) in out.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
                // new[r][c] = old[15 - c][r], as in main.swift
                *cell = g[15 - c][r];
            }
        }
        g = out;
    }
    g
}

/// Little pictures shown in speech bubbles in place of the Mac version's emoji.
/// Drawn at 2 pt per art pixel with `prop_palette`.
pub mod bubble_icons {
    pub const DROP: &[&str] = &[
        "...k...", "..kck..", ".kccck.", "kcCccck", "kcCccck", "kccccck", ".kkkkk.",
    ];
    pub const RICE: &[&str] = &[
        "..kkkkk..",
        ".kiiiiik.",
        "kiiiiiiik",
        "kkkkkkkkk",
        "koooooook",
        ".koooook.",
        "..kkkkk..",
    ];
    pub const STEW: &[&str] = &[
        "..g...g..",
        "...g...g.",
        "..g...g..",
        "kkkkkkkkk",
        "dkoyoNokd",
        ".kdddddk.",
        ".kdddddk.",
        "..kkkkk..",
    ];
    pub const STRETCH: &[&str] = &[
        "k.....k", ".k.o.k.", "..kok..", "...k...", "...k...", "..k.k..", ".k...k.",
    ];
    pub const TULIP: &[&str] = &["..P..", ".PyP.", "..P..", "..G..", ".GG..", "..G.."];
    pub const HEART: &[&str] = &[
        ".PP.PP.", "PPPPPPP", "PPPPPPP", ".PPPPP.", "..PPP..", "...P...",
    ];
}

/// Shifts the hue of the given palette keys (HSB, like NSColor), wrapping around.
pub fn hue_shifted(palette: &Palette, amount: f64, keys: &[char]) -> Palette {
    let mut out = *palette;
    for &key in keys {
        let Some(c) = palette.get(key as u8) else {
            continue;
        };
        let (r, g, b) = (c.r as f64 / 255.0, c.g as f64 / 255.0, c.b as f64 / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let v = max;
        let s = if max == 0.0 { 0.0 } else { d / max };
        let mut h = if d == 0.0 {
            0.0
        } else if max == r {
            ((g - b) / d) / 6.0
        } else if max == g {
            ((b - r) / d + 2.0) / 6.0
        } else {
            ((r - g) / d + 4.0) / 6.0
        };
        h = (h + amount).rem_euclid(1.0);
        let hh = h * 6.0;
        let i = hh.floor() as i32 % 6;
        let f = hh - hh.floor();
        let (p, q, t) = (v * (1.0 - s), v * (1.0 - f * s), v * (1.0 - (1.0 - f) * s));
        let (r, g, b) = match i {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };
        out.set(
            key,
            Some(Color::new(
                (r * 255.0).round() as u8,
                (g * 255.0).round() as u8,
                (b * 255.0).round() as u8,
                c.a,
            )),
        );
    }
    out
}
