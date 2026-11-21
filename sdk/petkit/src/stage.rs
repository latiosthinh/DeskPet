//! [`Stage`]: the pet's world as a trick sees it, and the verbs a trick uses.
//!
//! Coordinates are points, x from the pen's left edge and y up from the floor. The pen (the pet's
//! window) is `width`×`height` points; the pet's 16×16 sprite is [`PET_SIZE`] points wide and its left
//! edge is at `pet_x`. One art pixel is [`PIXEL`] points.

use crate::art::{rows, Color, Eyes, Frame, Pose, PET_SIZE, PIXEL, PROP_ROOM};
use crate::look::{free, pt, FreeSprite, Look, Particle, Pt};
use crate::pet::PetInfo;
use crate::random::rand;
use std::collections::{HashMap, HashSet};

/// What the pet is doing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Idle,
    Walking,
    Sitting,
    Typing,
    Cheering,
    Sleeping,
    Sweeping,
    Pastime,
    Drinking,
    Eating,
    Trick,
}

pub struct Stage {
    /// The pet's description.
    pub pet: PetInfo,

    pub mode: Mode,
    /// When the current mode (trick, activity) started and when it ends, in seconds of `now`.
    pub mode_start: f64,
    pub mode_until: f64,
    /// Left edge of the sprite, in points.
    pub pet_x: f64,
    /// Where a walk is heading.
    pub target_x: f64,
    pub facing_left: bool,

    /// The pen, in points.
    pub width: f64,
    pub height: f64,
    /// How far the pet roams, as set in the menu (only used to label the pen).
    pub range: f64,

    /// Seconds since the app started (`now`) and since this pet arrived (`clock`, for animations).
    pub now: f64,
    pub clock: f64,
    /// The pointer in pen coordinates.
    pub mouse: Pt,
    pub dragging: bool,

    pub particles: Vec<Particle>,
    pub bubble_text: Option<String>,
    pub bubble_icon: Option<&'static [&'static str]>,
    pub bubble_until: f64,
    pub show_pen_until: f64,
    pub home_moved: bool,

    // Trick bookkeeping (the engine keeps these; tricks read `trick_start_facing` and `trick_again`).
    #[doc(hidden)]
    pub trick: Option<usize>,
    #[doc(hidden)]
    pub trick_ids: Vec<&'static str>,
    #[doc(hidden)]
    pub last_trick: Option<usize>,
    #[doc(hidden)]
    pub trick_flags: HashSet<i32>,
    #[doc(hidden)]
    pub trick_timers: HashMap<i32, f64>,
    /// Which way the pet faced when the trick started.
    pub trick_start_facing: bool,
    /// Clicks waiting to play the click trick again (Moona throws her glaive once more per click).
    pub trick_again: u8,
    #[doc(hidden)]
    pub walk_after_trick: bool,
    #[doc(hidden)]
    pub restart_trick: bool,
    #[doc(hidden)]
    pub next_trick_at: Option<f64>,
    #[doc(hidden)]
    pub last_puff: f64,
}

impl Stage {
    pub fn new(pet: PetInfo, now: f64) -> Stage {
        Stage {
            pet,
            mode: Mode::Idle,
            mode_start: now,
            mode_until: now,
            pet_x: 0.0,
            target_x: 0.0,
            facing_left: false,
            width: 200.0,
            height: 134.0,
            range: 80.0,
            now,
            clock: 0.0,
            mouse: pt(-10000.0, -10000.0),
            dragging: false,
            particles: vec![],
            bubble_text: None,
            bubble_icon: None,
            bubble_until: 0.0,
            show_pen_until: 0.0,
            home_moved: false,
            trick: None,
            trick_ids: vec![],
            last_trick: None,
            trick_flags: HashSet::new(),
            trick_timers: HashMap::new(),
            trick_start_facing: false,
            trick_again: 0,
            walk_after_trick: false,
            restart_trick: false,
            next_trick_at: None,
            last_puff: 0.0,
        }
    }

    // MARK: Where things are

    pub fn min_pet_x(&self) -> f64 {
        PROP_ROOM
    }
    pub fn max_pet_x(&self) -> f64 {
        self.min_pet_x().max(self.width - PET_SIZE - PROP_ROOM)
    }
    /// Clickable area of the sprite: x, y (up from the floor), width, height.
    pub fn pet_rect(&self) -> (f64, f64, f64, f64) {
        if self.pet.picture.is_some() {
            let (w, h) = self.pet.size();
            return (self.pet_center_x() - w / 2.0, 0.0, w, h);
        }
        (
            self.pet_x + PIXEL,
            0.0,
            PET_SIZE - 2.0 * PIXEL,
            PET_SIZE - 2.0 * PIXEL,
        )
    }
    pub fn pet_center_x(&self) -> f64 {
        self.pet_x + PET_SIZE / 2.0
    }
    /// Left edge in points of a sprite column (0–15), honouring which way the pet faces.
    pub fn x_for_column(&self, col: f64) -> f64 {
        self.pet_x + (if self.facing_left { 15.0 - col } else { col }) * PIXEL
    }
    /// Centre x in points of a sprite column, honouring which way the pet faces.
    pub fn center_x(&self, col: f64) -> f64 {
        self.pet_x
            + (if self.facing_left {
                15.5 - col
            } else {
                col + 0.5
            }) * PIXEL
    }
    /// A velocity range pointing the way the pet faces.
    pub fn forward(&self, a: f64, b: f64) -> (f64, f64) {
        if self.facing_left {
            (-b, -a)
        } else {
            (a, b)
        }
    }
    /// A spot inside the pen within `reach` points of x, at least 20 points away when there's room.
    pub fn random_target(&self, x: f64, reach: f64) -> f64 {
        let lo = self.min_pet_x().max(x - reach);
        let hi = self.max_pet_x().min(x + reach);
        if hi <= lo {
            return x;
        }
        let mut t = x;
        for _ in 0..6 {
            t = rand(lo, hi);
            if (t - x).abs() > 20.0 {
                break;
            }
        }
        t
    }

    // MARK: Time

    /// How long the current trick or activity lasts.
    pub fn duration(&self) -> f64 {
        self.mode_until - self.mode_start
    }
    /// 0 or 1, flipping every `period` seconds.
    pub fn step_of(&self, period: f64) -> usize {
        ((self.clock / period) as i64 % 2) as usize
    }
    /// True and false in turn, flipping every `period` seconds.
    pub fn alt(&self, period: f64) -> bool {
        self.step_of(period) == 0
    }
    /// Open eyes with a blink now and then.
    pub fn blink_eyes(&self) -> Eyes {
        if self.clock % 4.2 < 0.14 {
            Eyes::Closed
        } else {
            Eyes::Open
        }
    }
    /// True the first time it's asked for each `id` during a trick: `if t > 2.0 && cx.once(1) { … }`.
    pub fn once(&mut self, id: i32) -> bool {
        self.trick_flags.insert(id)
    }
    /// True every `seconds` for each timer `id` during a trick.
    pub fn every(&mut self, id: i32, seconds: f64) -> bool {
        let last = self.trick_timers.get(&id).copied().unwrap_or(0.0);
        if self.now - last >= seconds {
            self.trick_timers.insert(id, self.now);
            return true;
        }
        false
    }

    // MARK: Doing things

    /// Shows a speech bubble for `seconds`.
    pub fn say(&mut self, text: &str, seconds: f64) {
        self.say_with(text, seconds, None);
    }
    /// A speech bubble with a little picture after the text.
    pub fn say_with(&mut self, text: &str, seconds: f64, icon: Option<&'static [&'static str]>) {
        self.bubble_text = Some(text.to_string());
        self.bubble_icon = icon;
        self.bubble_until = self.now + seconds;
    }
    /// Shows the pen's outline for a while.
    pub fn show_pen(&mut self, seconds: f64) {
        self.show_pen_until = self.show_pen_until.max(self.now + seconds);
    }
    /// A particle (a symbol such as "♥" or "✦") at `at`, moving with a velocity picked from the ranges,
    /// fading out over `life` seconds; `ay` is gravity (negative falls).
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_full(
        &mut self,
        text: &'static str,
        color: Color,
        at: Pt,
        size: f64,
        vx: (f64, f64),
        vy: (f64, f64),
        life: f64,
        ay: f64,
    ) {
        self.particles.push(Particle {
            text,
            color,
            size,
            x: at.x,
            y: at.y,
            vx: rand(vx.0, vx.1),
            vy: rand(vy.0, vy.1),
            life,
            max_life: life,
            ay,
        });
    }
    /// A particle that floats up and fades.
    pub fn spawn(&mut self, text: &'static str, color: Color, at: Pt, size: f64) {
        self.spawn_full(text, color, at, size, (-10.0, 10.0), (22.0, 36.0), 1.4, 0.0);
    }
    /// `count` particles floating up from above the pet's head.
    pub fn spawn_above_head(&mut self, text: &'static str, color: Color, count: usize, life: f64) {
        for _ in 0..count {
            let at = pt(
                self.pet_center_x() + rand(-14.0, 10.0),
                self.pet.size().1.max(PET_SIZE) - 14.0 + rand(-4.0, 4.0),
            );
            self.spawn_full(
                text,
                color,
                at,
                14.0,
                (-10.0, 10.0),
                (22.0, 36.0),
                life,
                0.0,
            );
        }
    }
    /// Walks toward x at `speed` points per second, turning to face it; true on arrival.
    pub fn move_toward(&mut self, x: f64, speed: f64, dt: f64) -> bool {
        let dx = x - self.pet_x;
        let stride = speed * dt;
        if dx.abs() <= stride {
            self.pet_x = x;
            return true;
        }
        self.facing_left = dx < 0.0;
        self.pet_x += if dx > 0.0 { stride } else { -stride };
        false
    }
    /// Plays the current trick again from the start once this frame is done (it doesn't count as
    /// one of the pet's own tricks).
    pub fn restart_trick(&mut self) {
        self.restart_trick = true;
    }
    /// The id of the trick being played, if any.
    pub fn trick_id(&self) -> Option<&'static str> {
        self.trick.and_then(|i| self.trick_ids.get(i).copied())
    }

    // MARK: Drawing helpers

    /// The pet sitting, blinking now and then: the starting point for most looks.
    pub fn look(&self) -> Look {
        Look::new(
            self.pet.frame(self.blink_eyes(), Pose::Sit),
            self.pet.palette,
        )
    }
    /// A grid placed relative to the pet: its left edge `dx` art pixels from the sprite's left edge
    /// (mirrored when the pet faces left) and its bottom `up` art pixels above the ground. Kept
    /// inside the pen. Drawn in prop colours ([`crate::art::prop_palette`]).
    pub fn placed(&self, grid: &[&str], dx: f64, up: f64) -> FreeSprite {
        let width = grid.first().map_or(0, |r| r.len()) as f64 * PIXEL;
        let x = if self.facing_left {
            self.pet_x + PET_SIZE - dx * PIXEL - width
        } else {
            self.pet_x + dx * PIXEL
        };
        let x = x.max(0.0).min((self.width - width).max(0.0));
        let grid = if self.facing_left {
            grid.iter().map(|r| r.chars().rev().collect()).collect()
        } else {
            rows(grid)
        };
        free(grid, x, up * PIXEL, PIXEL)
    }
    /// A copy of a 16×16 frame placed like [`placed`](Stage::placed), drawn in the pet's own colours
    /// (Kage's clones).
    pub fn frame_copy(&self, frame: &Frame, dx: f64, up: f64) -> FreeSprite {
        let lines: Vec<String> = frame
            .iter()
            .map(|r| r.iter().map(|&b| b as char).collect())
            .collect();
        let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        FreeSprite {
            own_colors: true,
            ..self.placed(&refs, dx, up)
        }
    }
    /// How high a held book sits: a pet that doesn't crouch to sit holds it lower, below its eyes.
    pub fn book_lift(&self) -> f64 {
        if self.pet.sit_base.is_none() {
            3.0
        } else {
            1.0
        }
    }
}
