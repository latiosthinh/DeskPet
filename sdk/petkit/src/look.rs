//! What a pet draws for one moment: its frame, the props it holds, sprites around it, particles.

use crate::art::{Color, Frame, Palette, Rows};

#[derive(Clone, Copy)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

pub fn pt(x: f64, y: f64) -> Pt {
    Pt { x, y }
}

/// Something the pet holds or uses, drawn in front of it.
/// `col` is the sprite column of the prop's left edge (facing right); `lift` is art pixels above the ground.
pub struct Prop {
    pub rows: Rows,
    pub col: f64,
    pub lift: f64,
}

pub fn prop(rows: Rows, col: f64, lift: f64) -> Prop {
    Prop { rows, col, lift }
}

/// A grid drawn at a fixed spot in the pen (in points), not attached to the pet.
pub struct FreeSprite {
    pub rows: Rows,
    pub x: f64,
    pub y: f64,
    pub px: f64,
    pub own_colors: bool, // drawn in the pet's colours rather than the prop colours (Moona's glaive and moonbeams)
}

pub fn free(rows: Rows, x: f64, y: f64, px: f64) -> FreeSprite {
    FreeSprite { rows, x, y, px, own_colors: false }
}

/// A second copy of the pet (Puddle's twin), centred at `center_x`.
pub struct Twin {
    pub frame: Frame,
    pub center_x: f64,
    pub scale: f64,
    pub y_offset: f64,
    pub facing_left: bool,
}

/// Everything drawn for one moment: the pet, how it is placed, and what is around it.
pub struct Look {
    pub frame: Frame,
    pub palette: Palette,
    pub y_offset: f64, // art pixels; negative sinks into water or soil
    pub scale: f64,
    pub props: Vec<Prop>,
    pub free: Vec<FreeSprite>,
    pub back: Vec<FreeSprite>, // drawn behind the pet
    pub x_offset: f64,         // art pixels forward (the way the pet faces), for jitters and recoils
    pub twin: Option<Twin>,
    pub beam: Option<(Pt, f64)>, // flashlight light
}

impl Look {
    pub fn new(frame: Frame, palette: Palette) -> Look {
        Look { frame, palette, y_offset: 0.0, scale: 1.0, props: vec![], free: vec![], back: vec![], x_offset: 0.0, twin: None, beam: None }
    }
}

#[derive(Clone)]
pub struct Particle {
    pub text: &'static str,
    pub color: Color,
    pub size: f64,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub life: f64,
    pub max_life: f64,
    pub ay: f64, // gravity (negative falls)
}
