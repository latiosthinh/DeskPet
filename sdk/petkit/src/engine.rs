//! The behaviour every pet shares: wandering about, sitting, sleeping, typing along with you,
//! reminders, and playing its tricks now and then. Pet authors don't need to touch this; the
//! app drives a [`Brain`] once per frame and draws what [`Brain::render`] returns.

use crate::art::{bubble_icons, props, sys_color, Color, Eyes, Frame, Picture, Pose, PET_SIZE};
use crate::kit;
use crate::look::{prop, pt, Look, Particle, Prop, Pt};
use crate::pet::{Category, Pastime, PetDef, Trick, Typing};
use crate::random::{rand, rand01, rand_index};
use crate::stage::{Mode, Stage};
use std::f64::consts::PI;

const WALK_SPEED: f64 = 42.0; // points per second
const SWEEP_SPEED: f64 = 12.0;
const SLEEP_AFTER: f64 = 90.0; // seconds of no input
const TYPING_LINGER: f64 = 3.0; // keep the laptop out this long after the last key

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reminder {
    Water,
    Lunch,
    Dinner,
    Stretch,
}

// MARK: The boundary with the app
//
// The app only ever talks to a pet through these: an [`Input`] each frame, a [`Command`] when
// something happens, [`About`] once, and a [`Render`] to draw. Nothing else crosses, so a pet can
// run sandboxed in its own module.

/// What the app tells the pet every frame.
#[derive(Clone, Copy)]
pub struct Input {
    /// Seconds since the last frame, and since the app started.
    pub dt: f64,
    pub now: f64,
    /// Seconds since any input, and since the last key press.
    pub idle: f64,
    pub key_idle: f64,
    /// The pointer in pen coordinates.
    pub mouse: Pt,
}

/// Something that happened, from the app's menus, the mouse or its timers.
#[derive(Clone)]
pub enum Command {
    /// The pen's size in points, and the roam range picked in the menu.
    Pen {
        width: f64,
        height: f64,
        range: f64,
    },
    /// Puts the pet at this x (kept inside the pen), e.g. after the pen moved.
    PlaceAt(f64),
    Say(String, f64),
    ShowPen(f64),
    /// Plays one of its tricks (by index), from the menu.
    Trick(usize),
    /// Plays a trick picked at random.
    SurpriseTrick,
    /// The pet has just been picked in the menu: says hello.
    Hello,
    Remind(Reminder),
    BeginDrag,
    /// The mouse button went up: `moved` says it was a drag, otherwise a click.
    EndDrag {
        moved: bool,
    },
    /// A right-click on the pet; [`Brain::command`] returns true if it did a trick.
    RightClick,
    /// Stops walking or sweeping (the pen just changed size).
    StopRoaming,
    /// The app saved the pet's new home.
    HomeSaved,
    /// The `--demo` flags: walk to the far side, or do an activity for a while.
    DemoWalk,
    DemoTyping(f64),
    DemoPastime(f64),
    DemoSweep(f64),
}

/// What the app needs to know about a pet, once: for its menus, its tray icon and sizing the pen.
#[derive(Clone)]
pub struct About {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub category: Category,
    pub description: String,
    pub author: String,
    pub version: u32,
    /// Titles and lengths of its tricks, in menu order.
    pub tricks: Vec<(String, f64)>,
    pub sweeps: bool,
    pub has_pastime: bool,
    /// Titles of the tricks a click and a right-click do, if they do one.
    pub click_trick: Option<String>,
    pub right_click_trick: Option<String>,
    /// Height in points of its tallest frame.
    pub tallest: f64,
    /// The pet standing, eyes open, for icons: rows of colours (None is see-through).
    pub icon: Vec<Vec<Option<Color>>>,
}

/// A picture pet's frame, ready to draw.
#[derive(Clone)]
pub struct PictureFrame {
    pub rows: Vec<Vec<u8>>,
    /// The column that stands at the pet's centre.
    pub anchor: f64,
    /// Points per art pixel.
    pub px: f64,
}

/// Everything the app draws for one frame, and the little it needs to know about where the pet is.
pub struct Render {
    pub look: Look,
    /// For a picture pet, the picture `look.frame` names.
    pub picture: Option<PictureFrame>,
    /// How far held props move forward (a picture pet's long body).
    pub prop_shift: f64,
    /// The pet's paw colour (props' 'w').
    pub paw: Option<Color>,
    pub pet_x: f64,
    pub min_pet_x: f64,
    pub max_pet_x: f64,
    pub facing_left: bool,
    /// The clickable area: x, y (up from the floor), width, height.
    pub pet_rect: (f64, f64, f64, f64),
    /// Height of the pet's head, where the speech bubble goes.
    pub head: f64,
    pub width: f64,
    pub range: f64,
    pub dragging: bool,
    pub show_pen: bool,
    pub particles: Vec<Particle>,
    pub bubble: Option<(String, Option<&'static [&'static str]>)>,
    /// Free to show a reminder: not asleep, not being dragged, nothing else being said.
    pub can_remind: bool,
    /// A drag just moved its home; the app saves it and replies with [`Command::HomeSaved`].
    pub home_moved: bool,
}

/// A pet brought to life: its stage plus its own tricks and typing.
pub struct Brain {
    pub stage: Stage,
    tricks: Vec<Box<dyn Trick>>,
    typing: Box<dyn Typing>,
    decorate: Option<fn(&Stage, &mut Look)>,
}

impl Brain {
    pub fn new(def: PetDef, now: f64) -> Brain {
        let mut stage = Stage::new(def.info, now);
        stage.trick_ids = def.tricks.iter().map(|t| t.id()).collect();
        Brain { stage, tricks: def.tricks, typing: def.typing, decorate: def.decorate }
    }

    // MARK: Tricks

    pub fn trick_count(&self) -> usize {
        self.tricks.len()
    }
    pub fn trick_title(&self, i: usize) -> &'static str {
        self.tricks[i].title()
    }
    pub fn trick_duration(&self, i: usize) -> f64 {
        self.tricks[i].duration()
    }
    pub fn trick_index(&self, id: &str) -> Option<usize> {
        self.tricks.iter().position(|t| t.id() == id)
    }

    pub fn start_trick(&mut self, i: usize) {
        let d = self.tricks[i].duration();
        let s = &mut self.stage;
        s.trick = Some(i);
        s.last_trick = Some(i);
        s.trick_flags.clear();
        s.trick_timers.clear();
        s.restart_trick = false;
        set_mode(s, Mode::Trick, d);
        s.next_trick_at = Some(s.now + d + rand(s.pet.trick_gap.0, s.pet.trick_gap.1));
        s.trick_start_facing = s.facing_left;
        self.tricks[i].start(&mut self.stage);
    }

    /// Starts a trick that isn't one of the pet's own picks (a click, a turn, waking up), so the
    /// timing of its next own trick stays as it was.
    fn start_trick_keeping_time(&mut self, i: usize) {
        let keep = self.stage.next_trick_at;
        self.start_trick(i);
        self.stage.next_trick_at = keep;
    }

    fn trick_by_id(&self, id: Option<&'static str>) -> Option<usize> {
        id.and_then(|id| self.trick_index(id))
    }

    /// Starts the pet's own activity for while you type.
    pub fn begin_typing(&mut self, seconds: f64) {
        set_mode(&mut self.stage, Mode::Typing, seconds);
        self.stage.trick_timers.clear();
        self.stage.trick_flags.clear();
        self.typing.begin(&mut self.stage);
    }

    // MARK: Behaviour

    fn decide_next(&mut self) {
        // A trick now and then: the first 1–3 minutes after launch (sooner if the pet's gap is shorter),
        // then the pet's trick_gap apart (4–8 minutes for most).
        if self.stage.next_trick_at.is_none() {
            let (lo, hi) = self.stage.pet.trick_gap;
            self.stage.next_trick_at = Some(self.stage.now + rand(60.0, 180.0).min(rand(lo, hi)));
        }
        if !self.tricks.is_empty() && self.stage.now >= self.stage.next_trick_at.unwrap() {
            let choices: Vec<usize> = (0..self.tricks.len()).filter(|&i| Some(i) != self.stage.last_trick).collect();
            let favorite = self.trick_by_id(self.stage.pet.favorite_trick).filter(|_| rand01() < 0.5);
            let t = if let Some(f) = favorite {
                f
            } else if choices.is_empty() {
                0
            } else {
                choices[rand_index(choices.len())]
            };
            self.start_trick(t);
            return;
        }
        let s = &mut self.stage;
        let can_walk = s.max_pet_x() - s.min_pet_x() > 24.0;
        let r = rand01();
        if r < 0.45 && can_walk {
            s.target_x = s.random_target(s.pet_x, f64::MAX);
            // Turning round to walk the other way: some pets do a trick first (Moona throws her glaive,
            // sometimes two or three times), then set off.
            let turning = s.target_x != s.pet_x && (s.target_x < s.pet_x) != s.facing_left;
            if let (true, Some(t)) = (turning, self.trick_by_id(self.stage.pet.turn_trick)) {
                self.stage.facing_left = !self.stage.facing_left;
                self.start_trick_keeping_time(t);
                let r = rand01();
                self.stage.trick_again = if r < 0.15 {
                    2
                } else if r < 0.45 {
                    1
                } else {
                    0
                };
                self.stage.walk_after_trick = true;
                return;
            }
            set_mode(&mut self.stage, Mode::Walking, 0.0);
        } else if r < 0.57 && s.pet.sweeps {
            set_mode(s, Mode::Sweeping, rand(6.0, 10.0));
            s.target_x = s.random_target(s.pet_x, 50.0);
        } else if r < 0.70 && s.pet.pastime.is_some() {
            set_mode(s, Mode::Pastime, rand(8.0, 14.0));
        } else if r < 0.90 {
            set_mode(s, Mode::Sitting, rand(3.0, 8.0));
        } else {
            set_mode(s, Mode::Idle, rand(2.0, 4.0));
        }
    }

    /// One frame. `idle` is seconds since any input, `key_idle` since the last key.
    pub fn step(&mut self, dt: f64, now: f64, idle: f64, key_idle: f64, mouse: Pt) {
        {
            let s = &mut self.stage;
            s.now = now;
            s.mouse = mouse;
            s.clock += dt;
            for p in &mut s.particles {
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.vy += p.ay * dt;
                p.life -= dt;
            }
            s.particles.retain(|p| p.life > 0.0);
            if s.bubble_text.is_some() && now > s.bubble_until {
                s.bubble_text = None;
            }
            if s.dragging {
                return;
            }
        }

        // React to what the human is doing.
        let wake = self.trick_by_id(self.stage.pet.wake_trick);
        let s = &self.stage;
        let busy_with_reminder =
            matches!(s.mode, Mode::Drinking | Mode::Eating | Mode::Cheering) || (s.mode == Mode::Trick && s.trick.is_some() && s.trick == wake); // typing doesn't cut it short
        if s.mode != Mode::Sleeping && idle > SLEEP_AFTER {
            set_mode(&mut self.stage, Mode::Sleeping, 0.0);
        } else if s.mode == Mode::Sleeping && idle < 1.0 {
            if let Some(t) = wake {
                self.start_trick_keeping_time(t); // Moona greets you with the moonlight
            } else {
                set_mode(&mut self.stage, Mode::Idle, 1.5);
                self.stage.say("!", 1.2);
            }
        } else if s.mode != Mode::Sleeping && !busy_with_reminder && key_idle < 0.6 {
            if s.mode != Mode::Typing {
                self.begin_typing(3.0);
            }
            self.stage.mode_until = now + TYPING_LINGER;
        }

        let t = now - self.stage.mode_start;
        match self.stage.mode {
            Mode::Walking => {
                let target = self.stage.target_x;
                if self.stage.move_toward(target, WALK_SPEED, dt) {
                    set_mode(&mut self.stage, Mode::Idle, rand(2.0, 5.0));
                }
            }
            Mode::Sweeping => {
                let s = &mut self.stage;
                if s.move_toward(s.target_x, SWEEP_SPEED, dt) {
                    s.target_x = s.random_target(s.pet_x, 50.0);
                }
                if now - s.last_puff > 0.35 {
                    s.last_puff = now;
                    let at = pt(s.x_for_column(19.0), 0.0);
                    s.spawn_full("·", sys_color::BROWN.with_alpha(0.8), at, 16.0, (-14.0, 14.0), (6.0, 16.0), 0.9, 0.0);
                }
                if now >= s.mode_until {
                    self.decide_next();
                }
            }
            Mode::Sleeping => {
                let s = &mut self.stage;
                if now - s.last_puff > 1.3 {
                    s.last_puff = now;
                    s.spawn_above_head("z", sys_color::SECONDARY_LABEL, 1, 2.2);
                }
            }
            Mode::Eating => {
                let s = &mut self.stage;
                if now - s.last_puff > 0.8 {
                    s.last_puff = now;
                    let at = pt(s.x_for_column(18.0), 18.0);
                    s.spawn_full("~", sys_color::SECONDARY_LABEL, at, 12.0, (-3.0, 3.0), (10.0, 16.0), 1.2, 0.0);
                }
                if now >= s.mode_until {
                    done_eating(s);
                }
            }
            Mode::Drinking => {
                if now >= self.stage.mode_until {
                    done_eating(&mut self.stage);
                }
            }
            Mode::Trick => {
                if let Some(i) = self.stage.trick {
                    self.tricks[i].step(&mut self.stage, t, dt);
                    if self.stage.restart_trick {
                        self.start_trick_keeping_time(i);
                    }
                }
                if self.stage.mode == Mode::Trick && now >= self.stage.mode_until {
                    self.stage.trick = None;
                    if self.stage.walk_after_trick {
                        set_mode(&mut self.stage, Mode::Walking, 0.0);
                    } else {
                        self.decide_next();
                    }
                }
            }
            Mode::Typing => {
                self.typing.step(&mut self.stage, t, dt);
                if now >= self.stage.mode_until {
                    self.decide_next();
                }
            }
            Mode::Idle | Mode::Sitting | Mode::Cheering | Mode::Pastime => {
                if now >= self.stage.mode_until {
                    self.decide_next();
                }
            }
        }
        let s = &mut self.stage;
        s.pet_x = s.pet_x.clamp(s.min_pet_x(), s.max_pet_x()); // never leave the pen

        // Look at the cursor when it comes close (not while using a prop).
        let cx = s.pet_center_x();
        if matches!(s.mode, Mode::Idle | Mode::Sitting | Mode::Cheering)
            && mouse.y < s.height + 160.0
            && (mouse.x - cx).abs() < 220.0
            && (mouse.x - cx).abs() > 6.0
        {
            s.facing_left = mouse.x < cx;
        }
    }

    pub fn set_mode(&mut self, m: Mode, seconds: f64) {
        set_mode(&mut self.stage, m, seconds);
    }

    pub fn petted(&mut self) {
        let s = &mut self.stage;
        s.bubble_text = None; // a click also dismisses a reminder
        set_mode(s, Mode::Cheering, 1.6);
        s.spawn_above_head("♥", sys_color::PINK, 3, 1.4);
    }

    pub fn remind(&mut self, reminder: Reminder) {
        let s = &mut self.stage;
        match reminder {
            Reminder::Water => {
                // A pet that doesn't drink (Moona) just waits with you while you do.
                set_mode(s, if s.pet.drinks { Mode::Drinking } else { Mode::Sitting }, 7.0);
                s.say_with("Time for some water", 9.0, Some(bubble_icons::DROP));
            }
            Reminder::Lunch => {
                set_mode(s, Mode::Eating, 9.0);
                s.say_with("Lunch time! Go eat", 12.0, Some(bubble_icons::RICE));
            }
            Reminder::Dinner => {
                set_mode(s, Mode::Eating, 9.0);
                s.say_with("Dinner time!", 12.0, Some(bubble_icons::STEW));
            }
            Reminder::Stretch => {
                set_mode(s, Mode::Cheering, 3.0);
                s.say_with("Stretch break?", 9.0, Some(bubble_icons::STRETCH));
            }
        }
    }

    /// Free to show a reminder: not asleep, not being dragged, nothing else being said.
    pub fn can_remind(&self) -> bool {
        let s = &self.stage;
        s.mode != Mode::Sleeping && !s.dragging && s.bubble_text.is_none()
    }

    // MARK: Mouse (the window moves itself; it tells the brain when a drag starts and ends)

    pub fn begin_drag(&mut self) {
        self.stage.dragging = true;
    }

    pub fn end_drag(&mut self, moved: bool) {
        self.stage.dragging = false;
        if moved {
            let s = &mut self.stage;
            set_mode(s, Mode::Idle, 2.0);
            s.show_pen(1.5);
            s.say("new home!", 1.5);
            s.home_moved = true;
        } else if let Some(t) = self.trick_by_id(self.stage.pet.click_trick) {
            let s = &mut self.stage;
            s.bubble_text = None; // still dismisses a reminder
            if s.mode == Mode::Trick && s.trick == Some(t) {
                s.walk_after_trick = false; // stays put for the ones you ask for
                s.trick_again = (s.trick_again + 1).min(3); // keeps going, one more each click
            } else {
                s.trick_again = 0;
                self.start_trick(t);
            }
        } else {
            self.petted();
        }
    }

    /// A right-click on the pet: true if the pet has its own right-click trick (Moona's moonlight);
    /// otherwise the menu opens.
    pub fn right_clicked(&mut self) -> bool {
        let Some(t) = self.trick_by_id(self.stage.pet.right_click_trick) else {
            return false;
        };
        self.stage.bubble_text = None;
        if !(self.stage.mode == Mode::Trick && self.stage.trick == Some(t)) {
            self.start_trick(t);
        }
        true
    }

    // MARK: Looks

    pub fn current_look(&self) -> Look {
        let s = &self.stage;
        let t = s.now - s.mode_start;
        let mut look = if let (Mode::Trick, false, Some(i)) = (s.mode, s.dragging, s.trick) {
            self.tricks[i].look(s, t)
        } else if s.mode == Mode::Typing && !s.dragging {
            self.typing.look(s, t)
        } else {
            let mut look = Look::new(s.current_frame(), s.pet.palette);
            if let Some(p) = s.current_prop() {
                look.props.push(p);
            }
            if s.mode == Mode::Pastime && !s.dragging {
                s.pastime_look(&mut look);
            }
            if s.mode == Mode::Walking && s.pet.floats {
                look.y_offset = (1.0 + (s.clock * 3.0).sin()).round();
            }
            look
        };
        if let Some(f) = self.decorate {
            f(s, &mut look);
        }
        look
    }

    // MARK: The app's side

    /// What to draw now.
    pub fn render(&self) -> Render {
        let s = &self.stage;
        let look = self.current_look();
        let picture = s.pet.picture.as_ref().map(|pic| {
            let i = pic.index_for(&look.frame);
            PictureFrame { rows: pic.frames[i].clone(), anchor: pic.anchors[i], px: pic.px }
        });
        Render {
            picture,
            prop_shift: s.pet.picture.as_ref().map_or(0.0, |pic| pic.prop_shift),
            paw: look.palette.get(s.pet.paw),
            look,
            pet_x: s.pet_x,
            min_pet_x: s.min_pet_x(),
            max_pet_x: s.max_pet_x(),
            facing_left: s.facing_left,
            pet_rect: s.pet_rect(),
            head: s.pet.size().1.max(PET_SIZE),
            width: s.width,
            range: s.range,
            dragging: s.dragging,
            show_pen: s.dragging || s.now < s.show_pen_until,
            particles: s.particles.clone(),
            bubble: s.bubble_text.clone().map(|t| (t, s.bubble_icon)),
            can_remind: self.can_remind(),
            home_moved: s.home_moved,
        }
    }

    /// One frame of time passing.
    pub fn tick(&mut self, input: Input) {
        self.step(input.dt, input.now, input.idle, input.key_idle, input.mouse);
    }

    /// Something happened. Returns true for a right-click the pet used for a trick.
    pub fn command(&mut self, c: Command) -> bool {
        match c {
            Command::Pen { width, height, range } => {
                let s = &mut self.stage;
                (s.width, s.height, s.range) = (width, height, range);
            }
            Command::PlaceAt(x) => {
                let s = &mut self.stage;
                s.pet_x = x.clamp(s.min_pet_x(), s.max_pet_x());
            }
            Command::Say(text, seconds) => self.stage.say(&text, seconds),
            Command::ShowPen(seconds) => self.stage.show_pen(seconds),
            Command::Trick(i) if i < self.tricks.len() => {
                self.stage.bubble_text = None;
                self.start_trick(i);
            }
            Command::Trick(_) => {}
            Command::SurpriseTrick if !self.tricks.is_empty() => {
                self.stage.bubble_text = None;
                self.start_trick(rand_index(self.tricks.len()));
            }
            Command::SurpriseTrick => {}
            Command::Hello => {
                self.stage.next_trick_at = None; // its own trick timing starts now
                self.petted();
                let hi = format!("hi, I'm {}!", self.stage.pet.short_name());
                self.stage.say(&hi, 2.5);
            }
            Command::Remind(r) => self.remind(r),
            Command::BeginDrag => self.begin_drag(),
            Command::EndDrag { moved } => self.end_drag(moved),
            Command::RightClick => return self.right_clicked(),
            Command::StopRoaming => {
                if matches!(self.stage.mode, Mode::Walking | Mode::Sweeping) {
                    self.set_mode(Mode::Idle, 0.0);
                }
            }
            Command::HomeSaved => self.stage.home_moved = false,
            Command::DemoWalk => {
                // To the far side of the pen.
                let s = &mut self.stage;
                let (lo, hi) = (s.min_pet_x(), s.max_pet_x());
                s.target_x = if s.pet_x - lo > hi - s.pet_x { lo } else { hi };
                set_mode(s, Mode::Walking, 0.0);
            }
            Command::DemoTyping(seconds) => self.begin_typing(seconds),
            Command::DemoPastime(seconds) => self.set_mode(Mode::Pastime, seconds),
            Command::DemoSweep(seconds) => {
                let s = &mut self.stage;
                set_mode(s, Mode::Sweeping, seconds);
                s.target_x = s.random_target(s.pet_x, 50.0);
            }
        }
        false
    }

    /// The pet's description, for the app's menus and icons.
    pub fn about(&self) -> About {
        let p = &self.stage.pet;
        let title = |id: Option<&str>| id.and_then(|id| self.trick_index(id)).map(|i| self.tricks[i].title().to_string());
        let rows: Vec<Vec<u8>> = match &p.picture {
            Some(pic) => pic.frames[if pic.idle_sits { 2 } else { 0 }].clone(),
            None => p.rows(Eyes::Open).iter().map(|r| r.to_vec()).collect(),
        };
        About {
            id: p.id.to_string(),
            name: p.name.to_string(),
            short_name: p.short_name().to_string(),
            category: p.category,
            description: p.description.to_string(),
            author: p.author.to_string(),
            version: p.version,
            tricks: self.tricks.iter().map(|t| (t.title().to_string(), t.duration())).collect(),
            sweeps: p.sweeps,
            has_pastime: p.pastime.is_some(),
            click_trick: title(p.click_trick),
            right_click_trick: title(p.right_click_trick),
            tallest: p.tallest(),
            icon: rows.iter().map(|r| r.iter().map(|&c| p.palette.get(c)).collect()).collect(),
        }
    }
}

fn set_mode(s: &mut Stage, m: Mode, seconds: f64) {
    if m != Mode::Trick {
        s.walk_after_trick = false;
    }
    s.mode = m;
    s.mode_start = s.now;
    s.mode_until = s.now + seconds;
}

fn done_eating(s: &mut Stage) {
    set_mode(s, Mode::Cheering, 1.2);
    s.spawn_above_head("♥", sys_color::PINK, 1, 1.4);
}

pub fn is_sipping(t: f64) -> bool {
    t % 1.6 > 1.0
}

/// The default typing look: sitting at a little laptop, paws tapping.
pub fn laptop_look(cx: &Stage, t: f64) -> Look {
    let mut look = cx.look();
    let laptop = if t < 0.3 { props::laptop_closed() } else { props::laptop_open(cx.alt(0.12), cx.alt(0.5)) };
    look.props.push(prop(laptop, 14.0, 0.0));
    look
}

impl Stage {
    fn current_frame(&self) -> Frame {
        let eyes = self.blink_eyes();
        let beat = (self.clock / 0.16) as i64 % 2 == 0;
        let t = self.now - self.mode_start;
        let p = &self.pet;
        if self.dragging {
            return p.frame(Eyes::Happy, Pose::Stand(self.step_of(0.1)));
        }
        match self.mode {
            Mode::Walking => match p.picture.as_ref().filter(|pic| !pic.walk.is_empty()) {
                Some(pic) => Picture::marker_index(pic.walk[(self.clock / 0.14) as usize % pic.walk.len()]),
                None => p.frame(Eyes::Open, Pose::Stand(self.step_of(0.18))),
            },
            Mode::Idle => p.frame(eyes, if p.picture.as_ref().is_some_and(|pic| pic.idle_sits) { Pose::Sit } else { Pose::Stand(0) }),
            Mode::Sitting | Mode::Typing | Mode::Pastime => p.frame(eyes, Pose::Sit),
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

    fn current_prop(&self) -> Option<Prop> {
        if self.dragging {
            return None;
        }
        let t = self.now - self.mode_start;
        let progress = if self.mode_until > self.mode_start { (t / (self.mode_until - self.mode_start)).min(1.0) } else { 0.0 };
        match self.mode {
            Mode::Sweeping => Some(prop(props::broom(self.alt(0.3)), 14.0, 0.0)),
            Mode::Drinking => Some(prop(props::cup(1.0 - progress), 14.0, if is_sipping(t) { 5.0 } else { 3.0 })),
            Mode::Eating if self.pet.picture.as_ref().is_some_and(|pic| pic.eat.is_some()) => None, // has its own food
            Mode::Eating => Some(prop(props::bowl(1.0 - progress), 14.0, 0.0)),
            _ => None,
        }
    }

    /// What a pet does now and then to pass the time, sitting down.
    fn pastime_look(&self, look: &mut Look) {
        use kit::*;
        let t = self.now - self.mode_start;
        let progress = (t / (self.mode_until - self.mode_start).max(1.0)).min(1.0);
        match self.pet.pastime {
            None => {}
            Some(Pastime::Book) => look.props.push(prop(props::book(t % 3.0 < 0.25), 13.0, self.book_lift())),
            Some(Pastime::Yoyo) => {
                let depth = (5.0 * (PI * ((t * 1.1) % 1.0)).sin()).round();
                let string = vec!["i"; depth as usize + 1];
                look.free.push(self.placed(&string, 15.0, 8.0 - depth));
                look.free.push(self.placed(PASTIME_YOYO, 14.0, 5.0 - depth));
            }
            Some(Pastime::Knit) => {
                // The scarf grows longer the longer it knits.
                let len = 1 + (progress * 4.0) as usize;
                let scarf: Vec<&str> = (0..len).map(|i| if i % 2 == 0 { "rrr" } else { "yyy" }).collect();
                look.free.push(self.placed(&scarf, 13.0, 5.0 - len as f64));
                look.free.push(self.placed(if self.alt(0.25) { PASTIME_NEEDLES_A } else { PASTIME_NEEDLES_B }, 12.0, 5.0));
            }
            Some(Pastime::Flower) => {
                look.free.push(self.placed(HERO_FLOWER, 15.0, 0.0));
                let wings = if self.alt(0.15) { PASTIME_BUTTERFLY_A } else { PASTIME_BUTTERFLY_B };
                look.free.push(self.placed(wings, (17.0 + 3.0 * (t * 1.3).cos()).round(), (8.0 + 2.0 * (t * 2.1).sin()).round()));
            }
            Some(Pastime::Juggle) => {
                for (k, ball) in [PASTIME_BALL_R, PASTIME_BALL_Y, PASTIME_BALL_C].into_iter().enumerate() {
                    let a = (t * 1.4 + k as f64 / 3.0) * 2.0 * PI;
                    look.free.push(self.placed(ball, (15.0 + 4.0 * a.cos()).round(), (10.0 + 5.0 * a.sin()).round()));
                }
            }
            Some(Pastime::Bubbles) => {
                look.props.push(prop(crate::art::rows(PASTIME_WAND), 13.0, 4.0));
                for k in 0..4 {
                    let q = (t * 0.45 + k as f64 * 0.25) % 1.0;
                    let dx = 17.0 + 7.0 * q + (q * 9.0 + k as f64).sin();
                    look.free.push(self.placed(PASTIME_BUBBLE, dx.round(), (9.0 + 14.0 * q).round()));
                }
            }
        }
    }
}
