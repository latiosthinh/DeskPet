//! The pet's behaviour and drawing, ported from PetView in main.swift.
//! The pen (the pet's window) is `width`×`height` points; the pet moves inside it.
//! Coordinates are points with y pointing up from the floor, as on the Mac.

use crate::art::{self, bubble_icons, props, sys_color, Color, Eyes, Frame, Palette, Pet, Picture, Pose, Rows, Trick, PET_SIZE, PIXEL, PROP_ROOM};
use crate::canvas::{Canvas, Font};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

// MARK: Random numbers (xorshift; nothing here needs more)

thread_local! {
    static SEED: Cell<u64> = Cell::new({
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        t | 1
    });
}

pub fn rand01() -> f64 {
    SEED.with(|s| {
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        (x >> 11) as f64 / (1u64 << 53) as f64
    })
}

pub fn rand(lo: f64, hi: f64) -> f64 {
    lo + rand01() * (hi - lo)
}

pub fn rand_index(n: usize) -> usize {
    ((rand01() * n as f64) as usize).min(n.saturating_sub(1))
}

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

#[derive(Clone, Copy, PartialEq)]
pub enum Reminder {
    Water,
    Lunch,
    Dinner,
    Stretch,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Idle,
    Walking,
    Sitting,
    Typing,
    Cheering,
    Sleeping,
    Sweeping,
    Reading,
    Drinking,
    Eating,
    Trick,
}

pub const TRICK_GAP: (f64, f64) = (4.0 * 60.0, 8.0 * 60.0);
const WALK_SPEED: f64 = 42.0; // points per second
const SWEEP_SPEED: f64 = 12.0;
const SLEEP_AFTER: f64 = 90.0; // seconds of no input
const TYPING_LINGER: f64 = 3.0; // keep the laptop out this long after the last key

pub struct PetView {
    pub mode: Mode,
    pub mode_start: f64,
    pub mode_until: f64,
    pub pet_x: f64,
    pub target_x: f64,
    pub facing_left: bool,
    pub pet: &'static Pet,

    // Trick state
    pub trick: Option<Trick>,
    pub last_trick: Option<Trick>,
    pub trick_flags: HashSet<i32>,
    pub trick_timers: HashMap<i32, f64>,
    pub trick_start_facing: bool,
    pub trick_again: u8, // clicks waiting to throw Moona's glaive again
    pub burst_cycle: i32,
    pub pool_left: f64,
    pub pool_width: f64,
    pub twin_center_x: f64,
    pub food_kind: usize,
    pub chase_moving: bool,
    pub bubble_scale: f64,
    pub dot_x: f64,
    /// When the pet may do its next trick on its own.
    pub next_trick_at: Option<f64>,

    pub range: f64,
    pub width: f64, // the pen, in points
    pub height: f64,

    pub clock: f64,
    pub now: f64,
    /// The pointer in pen coordinates, as of the last step.
    pub mouse: Pt,
    pub last_puff: f64,
    pub particles: Vec<Particle>,
    pub bubble_text: Option<String>,
    pub bubble_icon: Option<&'static [&'static str]>,
    pub bubble_until: f64,
    pub show_pen_until: f64,

    pub dragging: bool,
    pub home_moved: bool, // set when a drag ends somewhere new; the app saves it
}

impl PetView {
    pub fn new(pet: &'static Pet, now: f64) -> PetView {
        PetView {
            mode: Mode::Idle,
            mode_start: now,
            mode_until: now,
            pet_x: 0.0,
            target_x: 0.0,
            facing_left: false,
            pet,
            trick: None,
            last_trick: None,
            trick_flags: HashSet::new(),
            trick_timers: HashMap::new(),
            trick_start_facing: false,
            trick_again: 0,
            burst_cycle: -1,
            pool_left: 0.0,
            pool_width: 0.0,
            twin_center_x: 0.0,
            food_kind: 0,
            chase_moving: false,
            bubble_scale: 1.0,
            dot_x: 0.0,
            next_trick_at: None,
            range: 80.0,
            width: 200.0,
            height: 134.0,
            clock: 0.0,
            now,
            mouse: pt(-10000.0, -10000.0),
            last_puff: 0.0,
            particles: vec![],
            bubble_text: None,
            bubble_icon: None,
            bubble_until: 0.0,
            show_pen_until: 0.0,
            dragging: false,
            home_moved: false,
        }
    }

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
        (self.pet_x + PIXEL, 0.0, PET_SIZE - 2.0 * PIXEL, PET_SIZE - 2.0 * PIXEL)
    }
    pub fn pet_center_x(&self) -> f64 {
        self.pet_x + PET_SIZE / 2.0
    }
    /// Free to show a reminder: not asleep, not being dragged, nothing else being said.
    pub fn can_remind(&self) -> bool {
        self.mode != Mode::Sleeping && !self.dragging && self.bubble_text.is_none()
    }

    // MARK: Behaviour

    pub fn set_mode(&mut self, m: Mode, seconds: f64) {
        self.mode = m;
        self.mode_start = self.now;
        self.mode_until = self.now + seconds;
    }

    pub fn say(&mut self, text: &str, seconds: f64) {
        self.say_with(text, seconds, None);
    }

    pub fn say_with(&mut self, text: &str, seconds: f64, icon: Option<&'static [&'static str]>) {
        self.bubble_text = Some(text.to_string());
        self.bubble_icon = icon;
        self.bubble_until = self.now + seconds;
    }

    pub fn show_pen(&mut self, seconds: f64) {
        self.show_pen_until = self.show_pen_until.max(self.now + seconds);
    }

    /// Screen x of a sprite column, honouring which way the pet faces.
    pub fn x_for_column(&self, col: f64) -> f64 {
        self.pet_x + (if self.facing_left { 15.0 - col } else { col }) * PIXEL
    }

    #[allow(clippy::too_many_arguments)]
    pub fn spawn_full(&mut self, text: &'static str, color: Color, at: Pt, size: f64, vx: (f64, f64), vy: (f64, f64), life: f64, ay: f64) {
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

    pub fn spawn(&mut self, text: &'static str, color: Color, at: Pt, size: f64) {
        self.spawn_full(text, color, at, size, (-10.0, 10.0), (22.0, 36.0), 1.4, 0.0);
    }

    pub fn spawn_above_head(&mut self, text: &'static str, color: Color, count: usize, life: f64) {
        for _ in 0..count {
            let at = pt(self.pet_center_x() + rand(-14.0, 10.0), self.pet.size().1.max(PET_SIZE) - 14.0 + rand(-4.0, 4.0));
            self.spawn_full(text, color, at, 14.0, (-10.0, 10.0), (22.0, 36.0), life, 0.0);
        }
    }

    pub fn petted(&mut self) {
        self.bubble_text = None; // a click also dismisses a reminder
        self.set_mode(Mode::Cheering, 1.6);
        self.spawn_above_head("♥", sys_color::PINK, 3, 1.4);
    }

    pub fn remind(&mut self, reminder: Reminder) {
        match reminder {
            Reminder::Water => {
                // A pet that doesn't drink (Moona) just waits with you while you do.
                self.set_mode(if self.pet.drinks { Mode::Drinking } else { Mode::Sitting }, 7.0);
                self.say_with("Time for some water", 9.0, Some(bubble_icons::DROP));
            }
            Reminder::Lunch => {
                self.set_mode(Mode::Eating, 9.0);
                self.say_with("Lunch time! Go eat", 12.0, Some(bubble_icons::RICE));
            }
            Reminder::Dinner => {
                self.set_mode(Mode::Eating, 9.0);
                self.say_with("Dinner time!", 12.0, Some(bubble_icons::STEW));
            }
            Reminder::Stretch => {
                self.set_mode(Mode::Cheering, 3.0);
                self.say_with("Stretch break?", 9.0, Some(bubble_icons::STRETCH));
            }
        }
    }

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

    fn decide_next(&mut self) {
        // A trick now and then: the first 1–3 minutes after launch (sooner if the pet's gap is shorter),
        // then the pet's trick_gap apart (4–8 minutes for most).
        if self.next_trick_at.is_none() {
            let (lo, hi) = self.pet.trick_gap;
            self.next_trick_at = Some(self.now + rand(60.0, 180.0).min(rand(lo, hi)));
        }
        if !self.pet.tricks.is_empty() && self.now >= self.next_trick_at.unwrap() {
            let choices: Vec<Trick> = self.pet.tricks.iter().copied().filter(|t| Some(*t) != self.last_trick).collect();
            let favorite = self.pet.favorite_trick.filter(|_| rand01() < 0.5);
            let t = if let Some(f) = favorite {
                f
            } else if choices.is_empty() {
                self.pet.tricks[0]
            } else {
                choices[rand_index(choices.len())]
            };
            self.start_trick(t);
            return;
        }
        let can_walk = self.max_pet_x() - self.min_pet_x() > 24.0;
        let r = rand01();
        if r < 0.45 && can_walk {
            self.target_x = self.random_target(self.pet_x, f64::MAX);
            self.set_mode(Mode::Walking, 0.0);
        } else if r < 0.57 && self.pet.sweeps {
            self.set_mode(Mode::Sweeping, rand(6.0, 10.0));
            self.target_x = self.random_target(self.pet_x, 50.0);
        } else if r < 0.70 && self.pet.reads {
            self.set_mode(Mode::Reading, rand(8.0, 14.0));
        } else if r < 0.90 {
            self.set_mode(Mode::Sitting, rand(3.0, 8.0));
        } else {
            self.set_mode(Mode::Idle, rand(2.0, 4.0));
        }
    }

    pub fn step(&mut self, dt: f64, now: f64, idle: f64, key_idle: f64, mouse: Pt) {
        self.now = now;
        self.mouse = mouse;
        self.clock += dt;

        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += p.ay * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
        if self.bubble_text.is_some() && now > self.bubble_until {
            self.bubble_text = None;
        }

        if self.dragging {
            return;
        }

        // React to what the human is doing.
        let busy_with_reminder = matches!(self.mode, Mode::Drinking | Mode::Eating | Mode::Cheering);
        if self.mode != Mode::Sleeping && idle > SLEEP_AFTER {
            self.set_mode(Mode::Sleeping, 0.0);
        } else if self.mode == Mode::Sleeping && idle < 1.0 {
            self.set_mode(Mode::Idle, 1.5);
            self.say("!", 1.2);
        } else if self.mode != Mode::Sleeping && !busy_with_reminder && key_idle < 0.6 {
            if self.mode != Mode::Typing {
                self.begin_typing(3.0); // pull out the laptop, pan, shovel…
            }
            self.mode_until = now + TYPING_LINGER;
        }

        match self.mode {
            Mode::Walking => {
                if self.move_toward(self.target_x, WALK_SPEED, dt) {
                    self.set_mode(Mode::Idle, rand(2.0, 5.0));
                }
            }
            Mode::Sweeping => {
                if self.move_toward(self.target_x, SWEEP_SPEED, dt) {
                    self.target_x = self.random_target(self.pet_x, 50.0);
                }
                if now - self.last_puff > 0.35 {
                    self.last_puff = now;
                    let at = pt(self.x_for_column(19.0), 0.0);
                    self.spawn_full("·", sys_color::BROWN.with_alpha(0.8), at, 16.0, (-14.0, 14.0), (6.0, 16.0), 0.9, 0.0);
                }
                if now >= self.mode_until {
                    self.decide_next();
                }
            }
            Mode::Sleeping => {
                if now - self.last_puff > 1.3 {
                    self.last_puff = now;
                    self.spawn_above_head("z", sys_color::SECONDARY_LABEL, 1, 2.2);
                }
            }
            Mode::Eating => {
                if now - self.last_puff > 0.8 {
                    self.last_puff = now;
                    let at = pt(self.x_for_column(18.0), 18.0);
                    self.spawn_full("~", sys_color::SECONDARY_LABEL, at, 12.0, (-3.0, 3.0), (10.0, 16.0), 1.2, 0.0);
                }
                if now >= self.mode_until {
                    self.pet_done_eating();
                }
            }
            Mode::Drinking => {
                if now >= self.mode_until {
                    self.pet_done_eating();
                }
            }
            Mode::Trick => {
                if let Some(t) = self.trick {
                    self.step_trick(t, dt);
                }
                if now >= self.mode_until {
                    self.trick = None;
                    self.decide_next();
                }
            }
            Mode::Typing => {
                self.step_typing(dt);
                if now >= self.mode_until {
                    self.decide_next();
                }
            }
            Mode::Idle | Mode::Sitting | Mode::Cheering | Mode::Reading => {
                if now >= self.mode_until {
                    self.decide_next();
                }
            }
        }
        self.pet_x = self.pet_x.clamp(self.min_pet_x(), self.max_pet_x()); // never leave the pen

        // Look at the cursor when it comes close (not while using a prop).
        let cx = self.pet_center_x();
        if matches!(self.mode, Mode::Idle | Mode::Sitting | Mode::Cheering)
            && mouse.y < self.height + 160.0
            && (mouse.x - cx).abs() < 220.0
            && (mouse.x - cx).abs() > 6.0
        {
            self.facing_left = mouse.x < cx;
        }
    }

    fn pet_done_eating(&mut self) {
        self.set_mode(Mode::Cheering, 1.2);
        self.spawn_above_head("♥", sys_color::PINK, 1, 1.4);
    }

    /// Walks toward x; returns true on arrival.
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

    pub fn step_of(&self, period: f64) -> usize {
        ((self.clock / period) as i64 % 2) as usize
    }
    pub fn alt(&self, period: f64) -> bool {
        self.step_of(period) == 0
    }
    pub fn blink_eyes(&self) -> Eyes {
        if self.clock % 4.2 < 0.14 { Eyes::Closed } else { Eyes::Open }
    }

    fn current_frame(&self) -> Frame {
        let eyes = self.blink_eyes();
        let beat = (self.clock / 0.16) as i64 % 2 == 0;
        let t = self.now - self.mode_start;
        let p = self.pet;
        if self.dragging {
            return p.frame(Eyes::Happy, Pose::Stand(self.step_of(0.1)));
        }
        match self.mode {
            Mode::Walking => match p.picture.as_ref().filter(|pic| !pic.walk.is_empty()) {
                Some(pic) => Picture::marker_index(pic.walk[(self.clock / 0.14) as usize % pic.walk.len()]),
                None => p.frame(Eyes::Open, Pose::Stand(self.step_of(0.18))),
            },
            Mode::Idle => p.frame(eyes, if p.picture.as_ref().is_some_and(|pic| pic.idle_sits) { Pose::Sit } else { Pose::Stand(0) }),
            Mode::Sitting | Mode::Typing | Mode::Reading => p.frame(eyes, Pose::Sit),
            Mode::Cheering => p.frame(Eyes::Happy, if beat { Pose::Stand(0) } else { Pose::Sit }),
            Mode::Sleeping => match p.picture.as_ref().and_then(|pic| pic.sleep) {
                Some(i) => Picture::marker_index(i),
                None => p.frame(Eyes::Closed, Pose::Sit),
            },
            Mode::Sweeping => p.frame(eyes, Pose::Stand(self.step_of(0.3))),
            Mode::Drinking => p.frame(if is_sipping(t) { Eyes::Happy } else { Eyes::Open }, Pose::Sit),
            Mode::Eating => match p.picture.as_ref().and_then(|pic| pic.eat) {
                Some(i) => Picture::marker_index(i),
                None => p.frame(if self.step_of(0.4) == 0 { Eyes::Happy } else { Eyes::Open }, Pose::Sit),
            },
            Mode::Trick => p.frame(eyes, Pose::Sit),
        }
    }

    pub fn current_prop(&self) -> Option<Prop> {
        if self.dragging {
            return None;
        }
        let t = self.now - self.mode_start;
        let progress = if self.mode_until > self.mode_start { (t / (self.mode_until - self.mode_start)).min(1.0) } else { 0.0 };
        match self.mode {
            Mode::Typing => {
                if t < 0.3 {
                    return Some(prop(props::laptop_closed(), 14.0, 0.0));
                }
                Some(prop(props::laptop_open(self.alt(0.12), self.alt(0.5)), 14.0, 0.0))
            }
            Mode::Sweeping => Some(prop(props::broom(self.alt(0.3)), 14.0, 0.0)),
            Mode::Reading => Some(prop(props::book(t % 3.0 < 0.25), 13.0, self.book_lift())),
            Mode::Drinking => Some(prop(props::cup(1.0 - progress), 14.0, if is_sipping(t) { 5.0 } else { 3.0 })),
            Mode::Eating if self.pet.picture.as_ref().is_some_and(|pic| pic.eat.is_some()) => None, // has its own food
            Mode::Eating => Some(prop(props::bowl(1.0 - progress), 14.0, 0.0)),
            _ => None,
        }
    }

    pub fn current_look(&self) -> Look {
        let trick = if self.mode == Mode::Trick && !self.dragging { self.trick } else { None };
        let mut look = if let Some(t) = trick {
            self.trick_look(t)
        } else if self.mode == Mode::Typing && !self.dragging {
            self.typing_look()
        } else {
            let mut look = Look::new(self.current_frame(), self.pet.palette);
            if let Some(p) = self.current_prop() {
                look.props.push(p);
            }
            if self.mode == Mode::Reading && self.pet.reads_with_glasses && !self.dragging {
                look.free.push(self.glasses_sprite());
            }
            if self.mode == Mode::Walking && self.pet.floats {
                look.y_offset = (1.0 + (self.clock * 3.0).sin()).round();
            }
            look
        };
        // Sir Clank's key sits on his back, and turns during his wind-up trick.
        if self.pet.wind_key && trick != Some(Trick::ClankWind) {
            let key = self.placed(art::trick_art::HERO_KEY_A, -4.0 + look.x_offset, 4.0 + look.y_offset);
            look.back.push(key);
        }
        look
    }

    // MARK: Drawing

    pub fn draw(&mut self, c: &mut Canvas) {
        if self.dragging || self.now < self.show_pen_until {
            self.draw_pen(c);
        }

        let look = self.current_look();
        self.bubble_scale = look.scale; // a grown pet gets its speech bubble above its head
        if let Some((origin, angle)) = look.beam {
            self.draw_beam(c, origin, angle);
        }

        let mut colors = art::prop_palette();
        colors.set('w', look.palette.get(self.pet.paw));
        colors.set('k', look.palette.get(b'k'));

        for f in &look.back {
            let (x, y) = (c.snap(f.x), c.snap(f.y));
            draw_grid(c, &f.rows, if f.own_colors { &look.palette } else { &colors }, x, 0.0, y, f.px, false);
        }
        if let Some(tw) = &look.twin {
            draw_sprite(c, &tw.frame, &look.palette, tw.center_x, tw.scale, tw.y_offset, tw.facing_left);
        }
        let shift = look.x_offset * PIXEL * if self.facing_left { -1.0 } else { 1.0 };
        match &self.pet.picture {
            Some(pic) => draw_picture(c, pic, &look.frame, &look.palette, self.pet_center_x() + shift, look.scale, look.y_offset, self.facing_left),
            None => draw_sprite(c, &look.frame, &look.palette, self.pet_center_x() + shift, look.scale, look.y_offset, self.facing_left),
        }
        let px0 = c.snap(self.pet_x + shift);
        let prop_shift = self.pet.picture.as_ref().map_or(0.0, |pic| pic.prop_shift);
        for p in &look.props {
            draw_grid(c, &p.rows, &colors, px0, p.col + prop_shift, p.lift * PIXEL, PIXEL, self.facing_left);
        }
        for f in &look.free {
            let (x, y) = (c.snap(f.x), c.snap(f.y));
            draw_grid(c, &f.rows, if f.own_colors { &look.palette } else { &colors }, x, 0.0, y, f.px, false);
        }

        for p in &self.particles {
            let alpha = (p.life / p.max_life * 1.5).clamp(0.0, 1.0);
            c.text(p.text, Font::Symbol, p.size, p.color.with_alpha(alpha), p.x, p.y);
        }

        if let Some(text) = self.bubble_text.clone() {
            self.draw_bubble(c, &text, self.bubble_icon);
        }
    }

    fn draw_beam(&self, c: &mut Canvas, p: Pt, angle: f64) {
        let dir = if self.facing_left { -1.0 } else { 1.0 };
        let (length, spread) = (96.0, 0.34);
        let a = c.p(p.x, p.y);
        let b = c.p(p.x + dir * length * (angle - spread).cos(), p.y + length * (angle - spread).sin());
        let d = c.p(p.x + dir * length * (angle + spread).cos(), p.y + length * (angle + spread).sin());
        let light = Color::new(255, 237, 140, 128);
        c.fill_gradient_triangle(a, b, d, light, light.with_alpha(0.0), self.facing_left);
    }

    fn draw_pen(&self, c: &mut Canvas) {
        let (x, y) = (self.min_pet_x(), 1.0);
        let (w, h) = (self.max_pet_x() - self.min_pet_x() + PET_SIZE, PET_SIZE - 4.0);
        c.fill_round_rect(x, y, w, h, 10.0, sys_color::TEAL.with_alpha(0.12));
        c.stroke_round_rect(x, y, w, h, 10.0, sys_color::TEAL.with_alpha(0.8), 2.0, Some((6.0, 4.0)));

        let label = if self.range == 0.0 { "stays put".to_string() } else { format!("roams ±{} px", self.range as i32) };
        let (lw, _) = c.measure(&label, Font::Ui, 10.0);
        c.text(&label, Font::Ui, 10.0, sys_color::TEAL, x + w / 2.0 - lw / 2.0, y + h + 2.0);
    }

    fn draw_bubble(&self, c: &mut Canvas, text: &str, icon: Option<&'static [&'static str]>) {
        let icon_px = 2.0;
        let (tw, th) = c.measure(text, Font::Ui, 12.0);
        let (iw, ih) = match icon {
            Some(rows) => (rows.iter().map(|r| r.len()).max().unwrap_or(0) as f64 * icon_px, rows.len() as f64 * icon_px),
            None => (0.0, 0.0),
        };
        let content_w = tw + if icon.is_some() { 5.0 + iw } else { 0.0 };
        let content_h = th.max(ih);
        let cx = self.pet_center_x();

        let (bw, bh) = (content_w + 16.0, content_h + 8.0);
        let by = self.pet.size().1.max(PET_SIZE) * self.bubble_scale + 4.0;
        let bx = (cx - content_w / 2.0 - 8.0).max(2.0).min(self.width - bw - 2.0);

        let white = Color::new(255, 255, 255, 255);
        c.fill_round_rect(bx, by, bw, bh, 8.0, white);
        let (a, b, d) = (c.p(cx - 5.0, by + 1.0), c.p(cx, by - 6.0), c.p(cx + 5.0, by + 1.0));
        c.fill_triangle(a, b, d, white);
        c.stroke_round_rect(bx, by, bw, bh, 8.0, Color::new(51, 51, 51, 255), 1.5, None);
        c.text(text, Font::Ui, 12.0, Color::new(0, 0, 0, 255), bx + 8.0, by + 4.0 + (content_h - th) / 2.0);
        if let Some(rows) = icon {
            let ix = c.snap(bx + 8.0 + tw + 5.0);
            let iy = c.snap(by + 4.0 + (content_h - ih) / 2.0);
            draw_grid(c, rows, &art::prop_palette(), ix, 0.0, iy, icon_px, false);
        }
    }

    // MARK: Mouse (the window moves itself; it tells the view when a drag starts and ends)

    pub fn begin_drag(&mut self) {
        self.dragging = true;
    }

    pub fn end_drag(&mut self, moved: bool) {
        self.dragging = false;
        if moved {
            self.set_mode(Mode::Idle, 2.0);
            self.show_pen(1.5);
            self.say("new home!", 1.5);
            self.home_moved = true;
        } else if let Some(t) = self.pet.click_trick {
            self.bubble_text = None; // still dismisses a reminder
            if self.mode == Mode::Trick && self.trick == Some(t) {
                self.trick_again = (self.trick_again + 1).min(3); // keeps going, one more each click
            } else {
                self.trick_again = 0;
                self.start_trick(t);
            }
        } else {
            self.petted();
        }
    }

    /// A right-click on the pet: true if the pet has its own right-click trick (Moona's moonlight);
    /// otherwise the menu opens.
    pub fn right_clicked(&mut self) -> bool {
        let Some(t) = self.pet.right_click_trick else { return false };
        self.bubble_text = None;
        if !(self.mode == Mode::Trick && self.trick == Some(t)) {
            self.start_trick(t);
        }
        true
    }
}

pub fn is_sipping(t: f64) -> bool {
    t % 1.6 > 1.0
}

/// Draws the 16×16 pet sprite scaled around its centre, standing on the ground.
/// Art row 13 (feet) sits on the ground line, so the sprite starts 2 art pixels below it.
fn draw_sprite(c: &mut Canvas, rows: &Frame, palette: &Palette, center_x: f64, scale: f64, y_offset: f64, mirrored: bool) {
    let px = (PIXEL * scale * 2.0).round().max(1.0) / 2.0;
    let x = c.snap(center_x - 8.0 * px);
    let y = c.snap(-2.0 * px + y_offset * PIXEL);
    draw_grid(c, &rows[..], palette, x, 0.0, y, px, mirrored);
}

/// Draws a picture pet's frame (chosen by the marker in `frame`) scaled around its centre,
/// its bottom row standing on the ground.
#[allow(clippy::too_many_arguments)]
fn draw_picture(c: &mut Canvas, pic: &Picture, frame: &Frame, palette: &Palette, center_x: f64, scale: f64, y_offset: f64, mirrored: bool) {
    let i = pic.index_for(frame);
    let rows = &pic.frames[i];
    let px = (pic.px * scale * 2.0).round().max(1.0) / 2.0;
    let w = rows[0].len();
    let anchor = if mirrored { w as f64 - pic.anchors[i] } else { pic.anchors[i] };
    let x = c.snap(center_x - anchor * px);
    let bottom = c.snap(y_offset * PIXEL);
    let n = rows.len();
    for (r, row) in rows.iter().enumerate() {
        let y = bottom + (n - 1 - r) as f64 * px;
        for (i, &ch) in row.iter().enumerate() {
            let Some(color) = palette.get(ch) else { continue };
            let col = if mirrored { w - 1 - i } else { i } as f64;
            c.fill_edges(x + col * px, y, x + (col + 1.0) * px, y + px, color);
        }
    }
}

/// Draws pixel rows whose left edge is sprite column `col` and whose bottom row sits at `bottom_y`.
/// Mirrored grids flip around the 16-column sprite, so props stay in front of a pet facing left.
pub fn draw_grid<R: AsRef<[u8]>>(c: &mut Canvas, rows: &[R], palette: &Palette, x: f64, col: f64, bottom_y: f64, px: f64, mirrored: bool) {
    let n = rows.len();
    for (r, row) in rows.iter().enumerate() {
        let y = bottom_y + (n - 1 - r) as f64 * px;
        for (i, &ch) in row.as_ref().iter().enumerate() {
            let Some(color) = palette.get(ch) else { continue };
            let sprite_col = col + i as f64;
            let draw_col = if mirrored { 15.0 - sprite_col } else { sprite_col };
            c.fill_edges(x + draw_col * px, y, x + (draw_col + 1.0) * px, y + px, color);
        }
    }
}
