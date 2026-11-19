//! Each pet's own tricks and typing activities, ported from main.swift.

use crate::art::{bubble_icons, hue_shifted, props, rows, sys_color, trick_art, Eyes, Pose, Trick, TypingStyle, PET_SIZE, PIXEL};
use crate::pet::{free, prop, pt, rand, rand_index, Look, Mode, Particle, PetView, Pt, TRICK_GAP};
use std::f64::consts::PI;

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

impl PetView {
    /// Centre x of a sprite column, honouring which way the pet faces.
    fn center_x(&self, col: f64) -> f64 {
        self.pet_x + (if self.facing_left { 15.5 - col } else { col + 0.5 }) * PIXEL
    }

    /// Velocity range pointing the way the pet faces.
    fn forward(&self, a: f64, b: f64) -> (f64, f64) {
        if self.facing_left { (-b, -a) } else { (a, b) }
    }

    /// True once per trick for each id.
    fn once(&mut self, id: i32) -> bool {
        self.trick_flags.insert(id)
    }

    /// True every `seconds` for each timer id.
    fn every(&mut self, id: i32, seconds: f64) -> bool {
        let last = self.trick_timers.get(&id).copied().unwrap_or(0.0);
        if self.now - last >= seconds {
            self.trick_timers.insert(id, self.now);
            return true;
        }
        false
    }

    pub fn start_trick(&mut self, t: Trick) {
        self.trick = Some(t);
        self.last_trick = Some(t);
        self.trick_flags.clear();
        self.trick_timers.clear();
        self.set_mode(Mode::Trick, t.duration());
        self.next_trick_at = Some(self.now + t.duration() + rand(TRICK_GAP.0, TRICK_GAP.1));
        self.trick_start_facing = self.facing_left;
        match t {
            Trick::Swim => self.set_up_pool(),
            Trick::Laser => self.dot_x = self.pet_center_x(),
            Trick::Twin => {
                let room_right = self.width - (self.pet_x + PET_SIZE);
                let room_left = self.pet_x;
                let side = if room_right >= room_left { 1.0 } else { -1.0 };
                self.twin_center_x = (self.pet_center_x() + side * 68.0).max(PET_SIZE / 2.0).min(self.width - PET_SIZE / 2.0);
            }
            Trick::PopFood => self.food_kind = rand_index(trick_art::FOOD.len()),
            _ => {}
        }
    }

    fn swim_range(&self) -> (f64, f64) {
        let lo = (self.pool_left - 8.0).max(0.0);
        let hi = lo.max((self.width - PET_SIZE).min(self.pool_left + self.pool_width - PET_SIZE + 8.0));
        (lo, hi)
    }

    fn butterfly_point(&self, t: f64, d: f64) -> Pt {
        let lo: f64 = 16.0;
        let hi = (lo + 1.0).max(self.width - 16.0);
        let (mid, half) = ((lo + hi) / 2.0, ((hi - lo) / 2.0 - 10.0).max(0.0));
        let x = mid + half * (t * 0.8).sin() + 8.0 * (t * 3.1).sin();
        let mut y = 46.0 + 14.0 * (t * 2.2).sin();
        if t > d - 1.5 {
            y += (t - (d - 1.5)) * 70.0; // flies away at the end
        }
        pt(x, y)
    }

    fn firework(&mut self, p: Pt) {
        for i in 0..16 {
            let a = i as f64 / 16.0 * 2.0 * PI;
            let speed = rand(34.0, 52.0);
            self.particles.push(Particle {
                text: if i % 2 == 0 { "✦" } else { "•" },
                color: sys_color::SPARKLES[rand_index(6)],
                size: 11.0,
                x: p.x - 4.0,
                y: p.y,
                vx: a.cos() * speed,
                vy: a.sin() * speed,
                life: 1.3,
                max_life: 1.3,
                ay: -24.0,
            });
        }
    }

    // MARK: Trick motion and particles

    pub fn step_trick(&mut self, tr: Trick, dt: f64) {
        let t = self.now - self.mode_start;
        let d = self.mode_until - self.mode_start;
        let (blue, pink, grey) = (sys_color::BLUE, sys_color::PINK, sys_color::SECONDARY_LABEL);
        match tr {
            Trick::Firework => {
                let cycle = (t / 3.5) as i32;
                let p = t % 3.5;
                if p < 1.2 && self.every(1, 0.08) {
                    let at = pt(self.center_x(17.0) - 4.0, 22.0);
                    self.spawn_full("✦", sys_color::ORANGE, at, 9.0, (-10.0, 10.0), (4.0, 14.0), 0.45, 0.0);
                }
                if p >= 2.0 && self.burst_cycle != cycle {
                    self.burst_cycle = cycle;
                    let at = pt(self.center_x(17.0), 98.0);
                    self.firework(at);
                }
            }
            Trick::Flashlight => {
                self.facing_left = ((t / 2.7) as i32 % 2 == 1) != self.trick_start_facing;
                if self.every(1, 2.2) {
                    self.spawn_above_head("?", grey, 1, 1.4);
                }
            }
            Trick::FlipEgg => {
                if t % 2.0 < 0.7 && self.every(1, 0.3) {
                    let at = pt(self.center_x(19.0) - 4.0, 8.0 * PIXEL);
                    self.spawn_full("~", grey, at, 11.0, (-3.0, 3.0), (10.0, 16.0), 0.8, 0.0);
                }
            }
            Trick::Campfire => {
                if self.every(1, 0.45) {
                    let at = pt(self.center_x(18.0) - 4.0, 6.0 * PIXEL);
                    self.spawn_full("~", grey, at, 12.0, (-4.0, 4.0), (12.0, 20.0), 1.6, 0.0);
                }
                if self.every(2, 0.3) {
                    let at = pt(self.center_x(18.0) - 3.0, 5.0 * PIXEL);
                    self.spawn_full("·", sys_color::ORANGE, at, 16.0, (-10.0, 10.0), (18.0, 32.0), 0.8, 0.0);
                }
            }
            Trick::HideInWater => {
                if t > 0.8 && t < 3.5 && self.every(1, 0.4) {
                    let at = pt(self.pet_center_x() + rand(-12.0, 6.0), 6.0);
                    self.spawn_full("°", blue, at, 13.0, (-3.0, 3.0), (14.0, 22.0), 1.0, 0.0);
                }
                if t > 5.2 && self.once(1) {
                    self.facing_left = !self.facing_left;
                }
                if t > 6.4 && self.once(2) {
                    self.facing_left = !self.facing_left;
                }
                if t > 7.9 && self.once(3) {
                    for _ in 0..10 {
                        let at = pt(self.pet_center_x() - 4.0, 10.0);
                        self.spawn_full("•", blue, at, 10.0, (-55.0, 55.0), (35.0, 75.0), 0.9, -170.0);
                    }
                }
            }
            Trick::SpitWater => {
                if t % 0.9 < 0.1 && self.once(100 + (t / 0.9) as i32) {
                    for _ in 0..7 {
                        let at = pt(self.center_x(15.0), 5.5 * PIXEL);
                        let vx = self.forward(55.0, 95.0);
                        self.spawn_full("•", blue, at, 9.0, vx, (25.0, 55.0), 1.1, -170.0);
                    }
                }
            }
            Trick::Swim => self.swim_step(dt),
            Trick::Bubbles => {
                if self.every(1, 0.45) {
                    let at = pt(self.center_x(15.0), 5.5 * PIXEL);
                    let vx = self.forward(8.0, 22.0);
                    self.spawn_full("○", sys_color::TEAL, at, rand(9.0, 15.0), vx, (12.0, 24.0), 2.4, 0.0);
                }
            }
            Trick::IceCream => {
                if self.every(1, 1.8) {
                    self.spawn_above_head("♥", pink, 1, 1.4);
                }
            }
            Trick::Twin => {
                self.facing_left = self.twin_center_x < self.pet_center_x();
                if t > 0.6 && self.once(1) {
                    self.say("pop!", 1.0);
                }
                if t > 1.4 && self.every(1, 1.1) {
                    let at = pt((self.pet_center_x() + self.twin_center_x) / 2.0 - 4.0, PET_SIZE - 8.0);
                    self.spawn("♪", pink, at, 13.0);
                }
                if t > d - 0.6 && self.once(2) {
                    self.say("pop!", 1.0);
                }
            }
            Trick::ColorShift => {
                if self.every(1, 0.22) {
                    let at = pt(self.pet_center_x() + rand(-32.0, 26.0), rand(6.0, 50.0));
                    self.spawn_full("✦", sys_color::SPARKLES[rand_index(6)], at, 10.0, (-4.0, 4.0), (4.0, 10.0), 0.8, 0.0);
                }
            }
            Trick::PopFood => {
                if t > 1.2 && self.once(1) {
                    self.say("!", 0.8);
                }
                if t > 2.0 && self.once(2) {
                    self.say("a snack for you!", 3.0);
                    let at = pt(self.center_x(20.0) - 4.0, 12.0);
                    self.spawn_full("✦", sys_color::YELLOW, at, 11.0, (-10.0, 10.0), (22.0, 36.0), 0.9, 0.0);
                }
            }
            Trick::GiveFlower => {
                if self.once(1) {
                    self.say_with("for you!", 4.0, Some(bubble_icons::TULIP));
                }
                if t > 1.0 && self.every(1, 1.1) {
                    self.spawn_above_head("♥", pink, 1, 1.4);
                }
            }
            Trick::Shower => {
                let cloud_left = self.pet_center_x() - 20.0;
                if self.every(1, 0.06) {
                    let at = pt(cloud_left + rand(4.0, 34.0), 17.0 * PIXEL - 6.0);
                    self.spawn_full("|", blue.with_alpha(0.8), at, 9.0, (0.0, 0.0), (-95.0, -80.0), 0.65, 0.0);
                }
                if self.every(2, 1.4) {
                    let at = pt(self.pet_center_x() + 22.0, 40.0);
                    self.spawn("♪", pink, at, 13.0);
                }
            }
            Trick::PlantSelf => {
                if (t < 1.0 || (t > 7.5 && t < 8.3)) && self.every(1, 0.08) {
                    let at = pt(self.pet_center_x() + rand(-26.0, 20.0), 8.0);
                    self.spawn_full("·", sys_color::PLAIN_BROWN, at, 16.0, (-40.0, 40.0), (30.0, 60.0), 0.7, -150.0);
                }
                if t > 8.0 && self.once(1) {
                    self.say("!", 1.0);
                }
                if t > 8.3 && t < 9.3 {
                    self.facing_left = (t / 0.2) as i32 % 2 == 0; // shakes off the dirt
                }
            }
            Trick::GrowUp => {
                if ((t > 0.5 && t < 3.0) || (t > 6.0 && t < 7.0)) && self.every(1, 0.15) {
                    let at = pt(self.pet_center_x() + rand(-40.0, 34.0), rand(4.0, 70.0));
                    self.spawn_full("✦", sys_color::YELLOW, at, 10.0, (-4.0, 4.0), (4.0, 12.0), 0.8, 0.0);
                }
                if t > 3.0 && self.once(1) {
                    self.say("so big!", 2.0);
                }
            }
            Trick::ChaseButterfly => {
                let b = self.butterfly_point(t, d);
                let behind = if b.x > self.pet_center_x() { 16.0 } else { -16.0 };
                let target = (b.x - PET_SIZE / 2.0 - behind).max(0.0).min(self.width - PET_SIZE);
                self.chase_moving = !self.move_toward(target, 55.0, dt);
                if !self.chase_moving {
                    self.facing_left = b.x < self.pet_center_x();
                }
                if t > d - 1.5 && self.once(1) {
                    self.say("bye!", 1.5);
                }
            }
            Trick::Yarn => {
                let ball = self.yarn_x(t);
                let behind = if ball > self.pet_center_x() { 22.0 } else { -22.0 };
                let target = (ball - PET_SIZE / 2.0 - behind).max(0.0).min(self.width - PET_SIZE);
                self.chase_moving = !self.move_toward(target, 50.0, dt);
                if !self.chase_moving {
                    self.facing_left = ball < self.pet_center_x();
                }
                if t > 1.0 && self.every(1, 2.4) {
                    self.spawn_above_head("♪", pink, 1, 1.4);
                }
            }
            Trick::BoxNap => {
                if t > 1.4 && t < d - 1.4 && self.every(1, 1.3) {
                    self.spawn_above_head("z", grey, 1, 2.2);
                }
                if t > d - 1.2 && self.once(1) {
                    self.say("!", 1.0);
                }
            }
            Trick::Laser => {
                // The dot jumps every 1.6 s; Mochi crouches, wiggles, then pounces on it.
                let cycle = (t / 1.6) as i32;
                if self.once(200 + cycle) && cycle > 0 {
                    self.dot_x = rand(12.0, (self.width - 12.0).max(13.0));
                }
                if t % 1.6 > 0.55 {
                    let off = if self.dot_x > self.pet_center_x() { 10.0 } else { -10.0 };
                    let target = (self.dot_x - PET_SIZE / 2.0 - off).max(0.0).min(self.width - PET_SIZE);
                    self.chase_moving = !self.move_toward(target, 120.0, dt);
                } else {
                    self.chase_moving = false;
                    self.facing_left = self.dot_x < self.pet_center_x();
                }
            }
            Trick::Fishing => {
                let p = t % 3.0;
                if p > 2.0 && self.once(300 + (t / 3.0) as i32) {
                    for _ in 0..6 {
                        let at = pt(self.center_x(21.0) - 4.0, 12.0);
                        self.spawn_full("•", blue, at, 9.0, (-30.0, 30.0), (30.0, 55.0), 0.7, -160.0);
                    }
                }
                if p > 2.45 && self.once(400 + (t / 3.0) as i32) {
                    self.say("!", 0.6);
                }
            }
        }
    }

    // MARK: Typing activities

    /// Starts the pet's own activity for while you type (laptop, cooking, swimming, reading, digging).
    pub fn begin_typing(&mut self, seconds: f64) {
        self.set_mode(Mode::Typing, seconds);
        self.trick_timers.clear();
        self.trick_flags.clear();
        if self.pet.typing == TypingStyle::Swimming {
            self.set_up_pool();
        }
    }

    fn set_up_pool(&mut self) {
        self.pool_width = 30.0 * PIXEL;
        self.pool_left = (self.pet_center_x() - self.pool_width / 2.0).max(0.0).min((self.width - self.pool_width).max(0.0));
        self.target_x = self.swim_range().1;
    }

    fn swim_step(&mut self, dt: f64) {
        let (lo, hi) = self.swim_range();
        if self.move_toward(self.target_x, 22.0, dt) {
            self.target_x = if (self.target_x - lo).abs() < 1.0 { hi } else { lo };
        }
        if self.every(1, 0.35) {
            let at = pt(self.pet_center_x() + if self.facing_left { 22.0 } else { -30.0 }, 6.0);
            self.spawn_full("~", sys_color::BLUE, at, 12.0, (-4.0, 4.0), (2.0, 6.0), 0.8, 0.0);
        }
    }

    pub fn step_typing(&mut self, dt: f64) {
        match self.pet.typing {
            TypingStyle::Cooking => {
                if self.every(1, 0.35) {
                    let at = pt(self.center_x(20.0) - 4.0, 8.0 * PIXEL);
                    self.spawn_full("~", sys_color::SECONDARY_LABEL, at, 11.0, (-3.0, 3.0), (10.0, 16.0), 0.8, 0.0);
                }
            }
            TypingStyle::Swimming => self.swim_step(dt),
            TypingStyle::Digging => {
                if (self.clock / 0.3) as i64 % 2 == 1 && self.every(1, 0.3) {
                    for _ in 0..2 {
                        let at = pt(self.center_x(18.0) - 3.0, 8.0);
                        let vx = self.forward(-10.0, 40.0);
                        self.spawn_full("·", sys_color::PLAIN_BROWN, at, 16.0, vx, (30.0, 55.0), 0.7, -150.0);
                    }
                }
            }
            TypingStyle::Laptop | TypingStyle::ReadingGlasses => {}
        }
    }

    /// A pet that doesn't crouch to sit (Puddle) holds the book lower, below its eyes.
    pub fn book_lift(&self) -> f64 {
        if self.pet.sit_base.is_none() { 3.0 } else { 1.0 }
    }

    pub fn glasses_sprite(&self) -> crate::pet::FreeSprite {
        let half = PIXEL / 2.0;
        let start = 10.0;
        let width = trick_art::GLASSES[0].len() as f64;
        let grid = if self.facing_left {
            trick_art::GLASSES.iter().map(|r| r.chars().rev().collect()).collect()
        } else {
            rows(trick_art::GLASSES)
        };
        let x = self.pet_x + (if self.facing_left { 32.0 - start - width } else { start }) * half;
        free(grid, x, 11.0 * half, half)
    }

    pub fn typing_look(&self) -> Look {
        let t = self.now - self.mode_start;
        let eyes = self.blink_eyes();
        let p = self.pet;
        let mut look = Look::new(p.frame(eyes, Pose::Sit), p.palette);
        match p.typing {
            TypingStyle::Laptop => {
                if let Some(pr) = self.current_prop() {
                    look.props.push(pr);
                }
            }
            TypingStyle::Cooking => {
                // An egg in a pan over a little campfire, tossed now and then.
                let ph = t % 2.5;
                let mut egg = trick_art::EGG_UP;
                let mut lift = 7.0;
                if ph > 1.6 && ph < 2.3 {
                    lift = 7.0 + 6.0 * (PI * (ph - 1.6) / 0.7).sin();
                    egg = if self.alt(0.1) { trick_art::EGG_UP } else { trick_art::EGG_DOWN };
                }
                look.frame = p.frame(if ph > 2.3 { Eyes::Happy } else { eyes }, Pose::Stand(0));
                look.props.push(prop(rows(if self.alt(0.25) { trick_art::FIRE_A } else { trick_art::FIRE_B }), 17.0, 0.0));
                look.props.push(prop(rows(trick_art::PAN), 13.0, 5.0));
                look.props.push(prop(rows(egg), 18.0, lift));
            }
            TypingStyle::Swimming => {
                look.y_offset = -3.0;
                look.frame = p.frame(eyes, Pose::Stand(self.step_of(0.25)));
                look.free.push(free(trick_art::pool((self.pool_width / PIXEL) as usize), self.pool_left, 0.0, PIXEL));
            }
            TypingStyle::ReadingGlasses => {
                look.props.push(prop(props::book(t % 3.0 < 0.25), 13.0, self.book_lift()));
                look.free.push(self.glasses_sprite());
            }
            TypingStyle::Digging => {
                let down = !self.alt(0.3);
                let level = (1 + (t / 2.0) as usize).min(3);
                look.frame = p.frame(eyes, Pose::Stand(if down { 1 } else { 0 }));
                look.props.push(prop(rows(&trick_art::PILE[trick_art::PILE.len() - level..]), 15.0, 0.0));
                look.props.push(prop(rows(trick_art::SHOVEL), 13.0, if down { 1.0 } else { 3.0 }));
            }
        }
        look
    }

    fn yarn_x(&self, t: f64) -> f64 {
        let lo: f64 = 20.0;
        let hi = (lo + 1.0).max(self.width - 20.0);
        let (mid, half) = ((lo + hi) / 2.0, ((hi - lo) / 2.0 - 6.0).max(0.0));
        mid + half * (t * 0.9 + 0.6).sin()
    }

    // MARK: Trick looks

    pub fn trick_look(&self, tr: Trick) -> Look {
        let t = self.now - self.mode_start;
        let d = self.mode_until - self.mode_start;
        let eyes = self.blink_eyes();
        let p = self.pet;
        let mut look = Look::new(p.frame(eyes, Pose::Sit), p.palette);

        match tr {
            Trick::Firework => {
                let ph = t % 3.5;
                look.frame = p.frame(if ph > 2.0 { Eyes::Happy } else { eyes }, Pose::Sit);
                if ph < 1.2 {
                    look.props.push(prop(rows(trick_art::ROCKET), 16.0, 0.0));
                } else if ph < 2.0 {
                    look.props.push(prop(rows(trick_art::ROCKET), 16.0, (ph - 1.2) / 0.8 * 20.0));
                }
            }
            Trick::Flashlight => {
                look.frame = p.frame(eyes, Pose::Stand(0));
                look.props.push(prop(rows(trick_art::FLASHLIGHT), 14.0, 5.0));
                let origin = pt(self.center_x(18.0) + if self.facing_left { -2.0 } else { 2.0 }, 6.5 * PIXEL);
                look.beam = Some((origin, 0.05 + 0.22 * (t * 1.6).sin()));
            }
            Trick::FlipEgg => {
                let ph = t % 2.0;
                let mut egg = trick_art::EGG_UP;
                let mut lift = 7.0;
                if (0.7..1.6).contains(&ph) {
                    lift = 7.0 + 10.0 * (PI * (ph - 0.7) / 0.9).sin();
                    egg = if self.alt(0.1) { trick_art::EGG_UP } else { trick_art::EGG_DOWN };
                }
                look.frame = p.frame(if ph >= 1.6 { Eyes::Happy } else { eyes }, Pose::Stand(0));
                look.props.push(prop(rows(trick_art::PAN), 13.0, 5.0));
                look.props.push(prop(rows(egg), 18.0, lift));
            }
            Trick::Campfire => {
                look.frame = p.frame(if t > d - 2.0 { Eyes::Happy } else { eyes }, Pose::Sit);
                look.props.push(prop(rows(if self.alt(0.25) { trick_art::FIRE_A } else { trick_art::FIRE_B }), 15.0, 0.0));
                look.props.push(prop(trick_art::stick(t > d * 0.5), 13.0, 3.0));
            }
            Trick::HideInWater => {
                // Sink, wait under water, peek out with just the eyes, then jump out with a splash.
                let y = if t < 0.8 {
                    -12.0 * t / 0.8
                } else if t < 3.5 {
                    -12.0
                } else if t < 4.3 {
                    -12.0 + 8.0 * (t - 3.5) / 0.8
                } else if t < 7.5 {
                    -4.0
                } else if t < 7.9 {
                    -4.0 + 7.0 * (t - 7.5) / 0.4
                } else if t < 8.2 {
                    3.0 - 3.0 * (t - 7.9) / 0.3
                } else {
                    0.0
                };
                look.y_offset = y;
                look.frame = p.frame(if t > 7.5 { Eyes::Happy } else { eyes }, Pose::Stand(0));
                if t < 8.6 {
                    look.props.push(prop(trick_art::pool(20), -2.0, 0.0));
                }
            }
            Trick::SpitWater => {
                look.frame = p.frame(if t % 0.9 < 0.2 { Eyes::Happy } else { Eyes::Open }, Pose::Sit);
            }
            Trick::Swim => {
                look.y_offset = -3.0;
                look.frame = p.frame(Eyes::Open, Pose::Stand(self.step_of(0.25)));
                look.free.push(free(trick_art::pool((self.pool_width / PIXEL) as usize), self.pool_left, 0.0, PIXEL));
            }
            Trick::Bubbles => {
                look.frame = p.frame(if self.alt(0.45) { Eyes::Happy } else { Eyes::Open }, Pose::Sit);
            }
            Trick::IceCream => {
                look.frame = p.frame(if self.alt(0.5) { Eyes::Happy } else { Eyes::Open }, Pose::Sit);
                look.props.push(prop(trick_art::ice_cream(1.0 - t / d), 14.0, 3.0));
            }
            Trick::Twin => {
                let (start, end) = (0.6, d - 1.2);
                if t < start {
                    look.frame = p.frame(Eyes::Closed, Pose::Sit);
                    return look;
                }
                let hop = ((t - start) * 2.0 * PI / 0.8).sin();
                look.frame = p.frame(Eyes::Happy, Pose::Stand(0));
                look.y_offset = hop.max(0.0) * 3.0;
                let mut scale = 1.0;
                let mut cx = self.twin_center_x;
                if t < start + 0.5 {
                    scale = 0.4 + 0.6 * ((t - start) / 0.5);
                    cx = lerp(self.pet_center_x(), self.twin_center_x, (t - start) / 0.5);
                } else if t > end {
                    let q = (t - end) / 0.6;
                    scale = 1.0 - 0.6 * q.min(1.0);
                    cx = lerp(self.twin_center_x, self.pet_center_x(), q);
                }
                if t < end + 0.6 {
                    look.twin = Some(crate::pet::Twin {
                        frame: p.frame(Eyes::Happy, Pose::Stand(0)),
                        center_x: cx,
                        scale,
                        y_offset: (-hop).max(0.0) * 3.0,
                        facing_left: self.twin_center_x > self.pet_center_x(),
                    });
                }
                if t > start + 0.9 && t < end - 0.2 {
                    // a ball passed back and forth
                    let f = ((t - start - 0.9) / 1.2) % 2.0;
                    let q = if f < 1.0 { f } else { 2.0 - f };
                    let x = lerp(self.pet_center_x(), self.twin_center_x, q) - 4.0;
                    look.free.push(free(rows(trick_art::BALL), x, 30.0 + 26.0 * (PI * q).sin(), PIXEL));
                }
            }
            Trick::ColorShift => {
                look.palette = hue_shifted(&p.palette, t / d, &['c', 'd', 'k']);
                look.frame = p.frame(Eyes::Happy, Pose::Stand(if self.alt(0.5) { 0 } else { 1 }));
            }
            Trick::PopFood => {
                if t < 1.2 {
                    look.frame = p.frame(Eyes::Closed, Pose::Stand(self.step_of(0.12)));
                } else {
                    look.frame = p.frame(Eyes::Happy, Pose::Sit);
                    let q = ((t - 1.2) / 0.8).min(1.0);
                    look.props.push(prop(rows(trick_art::FOOD[self.food_kind]), 6.0 + 13.0 * q, (1.0 - q) * 10.0 + 12.0 * (PI * q).sin()));
                }
            }
            Trick::GiveFlower => {
                look.frame = p.frame(Eyes::Happy, Pose::Stand(0));
                look.props.push(prop(rows(trick_art::FLOWER), 14.0, if self.alt(0.6) { 4.0 } else { 5.0 }));
            }
            Trick::Shower => {
                look.frame = p.frame(Eyes::Happy, if self.alt(0.4) { Pose::Stand(0) } else { Pose::Sit });
                look.free.push(free(rows(trick_art::CLOUD), self.pet_center_x() - 20.0, 17.0 * PIXEL, PIXEL));
            }
            Trick::PlantSelf => {
                // Dig, sink until only the leaves show, nap, then pop out and shake off.
                let y = if t < 1.0 {
                    0.0
                } else if t < 2.5 {
                    -8.0 * (t - 1.0) / 1.5
                } else if t < 7.5 {
                    -8.0
                } else if t < 8.0 {
                    -8.0 + 10.0 * (t - 7.5) / 0.5
                } else if t < 8.3 {
                    2.0 - 2.0 * (t - 8.0) / 0.3
                } else {
                    0.0
                };
                look.y_offset = y;
                let e = if t > 2.5 && t < 7.5 { Eyes::Closed } else { Eyes::Open };
                look.frame = p.frame(e, Pose::Stand(if t < 1.0 { self.step_of(0.15) } else { 0 }));
                if t < 9.2 {
                    look.props.push(prop(rows(trick_art::SOIL), -2.0, 0.0));
                }
            }
            Trick::GrowUp => {
                let scale = if t < 1.0 {
                    1.0
                } else if t < 3.0 {
                    1.0 + 0.125 * ((((t - 1.0) / 0.5) as i32 + 1).min(4)) as f64
                } else if t < 6.0 {
                    1.5
                } else if t < 7.0 {
                    1.5 - 0.125 * ((((t - 6.0) / 0.25) as i32 + 1).min(4)) as f64
                } else {
                    1.0
                };
                look.scale = scale;
                look.frame = p.frame(if scale > 1.0 { Eyes::Happy } else { eyes }, Pose::Stand(0));
            }
            Trick::ChaseButterfly => {
                look.frame = p.frame(Eyes::Happy, Pose::Stand(if self.chase_moving { self.step_of(0.14) } else { 0 }));
                let b = self.butterfly_point(t, d);
                let art = if self.alt(0.15) { trick_art::BUTTERFLY_A } else { trick_art::BUTTERFLY_B };
                look.free.push(free(rows(art), b.x - 7.5, b.y, 3.0));
            }
            Trick::Yarn => {
                look.frame = p.frame(Eyes::Happy, Pose::Stand(if self.chase_moving { self.step_of(0.14) } else { 0 }));
                let x = self.yarn_x(t);
                let rolling = (x / 6.0) as i64 % 2 == 0;
                look.free.push(free(rows(if rolling { trick_art::YARN_A } else { trick_art::YARN_B }), x - 8.0, 0.0, PIXEL));
            }
            Trick::BoxNap => {
                // Hop into the box, nap with just the head showing, then pop up.
                let y = if t < 0.8 { 2.0 + 8.0 * (PI * t / 0.8).sin() } else if t > d - 1.2 { 3.0 } else { 2.0 };
                look.y_offset = y;
                look.frame = p.frame(if t > 1.2 && t < d - 1.2 { Eyes::Closed } else { Eyes::Happy }, Pose::Sit);
                look.props.push(prop(rows(trick_art::BOX), -1.0, 0.0));
            }
            Trick::Laser => {
                let ph = t % 1.6;
                if self.chase_moving {
                    look.frame = p.frame(Eyes::Open, Pose::Stand(0));
                    look.y_offset = 4.0 * (PI * ((ph - 0.55) / 0.5).min(1.0)).sin();
                } else {
                    look.frame = p.frame(Eyes::Open, if self.alt(0.12) { Pose::Sit } else { Pose::Stand(0) }); // wiggles before pouncing
                }
                look.free.push(free(rows(trick_art::LASER_DOT), self.dot_x - 2.5, 0.0, 5.0));
            }
            Trick::Fishing => {
                let ph = t % 3.0;
                look.frame = p.frame(if ph > 2.3 { Eyes::Happy } else { eyes }, Pose::Sit);
                look.props.push(prop(trick_art::pool(10), 15.0, 0.0));
                look.props.push(prop(rows(trick_art::ROD), 13.0, 3.0));
                if ph > 2.0 && ph < 2.9 {
                    let q = (ph - 2.0) / 0.9;
                    look.props.push(prop(rows(trick_art::FISH), 18.0, 3.0 + 9.0 * (PI * q).sin()));
                }
            }
        }
        look
    }
}
