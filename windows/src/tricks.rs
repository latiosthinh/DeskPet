//! Each pet's own tricks and typing activities, ported from main.swift.

use crate::art::{bubble_icons, hex, hue_shifted, pictures, props, rotated, rows, sys_color, trick_art, Eyes, Picture, Pose, Rows, Trick, TypingStyle, GLAIVE_THROW, PET_SIZE, PIXEL};
use crate::pet::{free, prop, pt, rand, rand_index, FreeSprite, Look, Mode, Particle, PetView, Pt};
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
        self.next_trick_at = Some(self.now + t.duration() + rand(self.pet.trick_gap.0, self.pet.trick_gap.1));
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

            Trick::GrumboSpin => {
                if t > 3.0 && self.once(1) {
                    self.say("whoa…", 1.5);
                }
            }
            Trick::GrumboHelmet => {
                if t > 1.0 && self.once(1) {
                    self.say("who turned off the lights?", 2.5);
                }
            }
            Trick::GrumboChop => {
                if t > 4.0 && self.once(1) {
                    self.say("boing!", 1.0);
                }
            }

            Trick::WispThread => {
                let target = self.thread_target(t);
                if (target.x - self.pet_center_x()).abs() > 6.0 {
                    self.facing_left = target.x < self.pet_center_x();
                }
            }
            Trick::WispKnot => {
                if t > 3.0 && self.once(1) {
                    self.say("oops, knotted", 1.8);
                }
            }
            Trick::WispMoth => {
                if t > 7.0 && self.once(1) {
                    self.say("!", 0.8);
                }
            }

            Trick::HexFrog => {
                if t > 3.0 && self.once(1) {
                    self.say("ribbit?!", 1.5);
                }
            }
            Trick::HexSheep => {
                if t > 3.0 && self.once(1) {
                    self.say("baa…", 1.5);
                }
            }
            Trick::HexBook => {
                if t > 3.0 && self.once(1) {
                    self.say("ow!", 1.0);
                }
            }

            Trick::MossToss | Trick::MossBloom => {}
            Trick::MossRain => {
                if t > 6.0 && self.every(1, 1.2) {
                    self.spawn_above_head("♥", pink, 1, 1.4);
                }
            }

            Trick::ClankWind => {
                // Slows to a stop, waits for his key to turn, then hurries off happily.
                let dir = if self.facing_left { -1.0 } else { 1.0 };
                if t < 3.0 {
                    self.pet_x += dir * 30.0 * (-t).exp() * dt;
                }
                if t > 7.0 {
                    self.pet_x += dir * 36.0 * dt;
                }
                if t > 5.0 && self.once(1) {
                    self.say("click… click…", 1.8);
                }
            }
            Trick::ClankSpring => {
                if t > 2.0 && self.once(1) {
                    self.say("achoo!", 1.2);
                }
            }
            Trick::ClankBell => {
                if t > 3.0 && self.once(1) {
                    self.say("tea time!", 2.2);
                }
            }

            Trick::LunetteRice => {
                if t > 5.0 && self.once(1) {
                    self.say("ta-da!", 1.8);
                }
            }
            Trick::LunetteWish => {
                if t > 4.0 && self.once(1) {
                    self.say("a little wish…", 2.0);
                }
            }
            Trick::LunetteNap => {}

            Trick::MoonaPounce => {
                // Leaps forward.
                if (POUNCE_UP..POUNCE_LAND).contains(&t) {
                    let dir = if self.trick_start_facing { -1.0 } else { 1.0 };
                    self.pet_x += dir * 60.0 * dt;
                }
            }
            Trick::MoonaGlaive => {
                // Another click came in while the glaive was out: throw it again once it's back.
                if t >= GLAIVE_THROW - 0.05 && self.trick_again > 0 {
                    self.trick_again -= 1;
                    let keep = self.next_trick_at;
                    self.start_trick(Trick::MoonaGlaive);
                    self.next_trick_at = keep;
                }
            }
            Trick::WitcherWhirl => {
                if t > ORB_WHIRL_START && self.once(1) {
                    self.say("orbs, unite!", 1.4);
                }
            }
            Trick::WitcherWeigh => {
                if t > 0.4 && self.once(1) {
                    self.say("hmm…", 1.6);
                }
            }
            Trick::WitcherVanish => {
                // Gone: he turns up somewhere else in the pen.
                if t > VANISH_MOVE && self.once(1) {
                    let from = self.pet_x;
                    self.pet_x = self.random_target(from, f64::MAX);
                    self.facing_left = self.pet_x > from;
                }
            }
            Trick::WitcherOrb => {}
            Trick::MoonaTriplicate => {
                if t > 0.4 && self.once(1) {
                    self.say("Manta style!", 1.4);
                }
            }
            Trick::MoonaMoonlight => {
                if t > 0.4 && self.once(1) {
                    self.say("moonlight!", 1.6);
                }
            }
        }
    }

    /// A grid placed like in prototypes/hero-pets.html: its left edge `dx` art pixels from the sprite's
    /// left edge (mirrored when the pet faces left) and its bottom `up` art pixels above the ground.
    /// It is kept inside the pen.
    pub fn placed(&self, grid: &[&str], dx: f64, up: f64) -> FreeSprite {
        let width = grid.first().map_or(0, |r| r.len()) as f64 * PIXEL;
        let x = if self.facing_left { self.pet_x + PET_SIZE - dx * PIXEL - width } else { self.pet_x + dx * PIXEL };
        let x = x.max(0.0).min((self.width - width).max(0.0));
        let grid = if self.facing_left { grid.iter().map(|r| r.chars().rev().collect()).collect() } else { rows(grid) };
        free(grid, x, up * PIXEL, PIXEL)
    }

    /// Where Wisp-a-ling's glowing thread reaches: the pointer when it is near, otherwise a spot up ahead.
    fn thread_target(&self, t: f64) -> Pt {
        let edge = 8.0;
        let mut p = pt(self.center_x(24.0) + 5.0 * PIXEL * t.sin(), (24.0 + 5.0 * t.cos()) * PIXEL);
        if self.mouse.y < self.height + 160.0 && (self.mouse.x - self.pet_center_x()).abs() < 320.0 {
            p = self.mouse;
        }
        pt(p.x.max(edge).min(self.width - edge), p.y.max(edge).min(self.height - edge))
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
            TypingStyle::Laptop
            | TypingStyle::ReadingGlasses
            | TypingStyle::Stamping
            | TypingStyle::Spool
            | TypingStyle::SpellNotes
            | TypingStyle::Pebbles
            | TypingStyle::Checklist
            | TypingStyle::Mortar
            | TypingStyle::Moonlight
            | TypingStyle::Orbs => {}
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
            TypingStyle::Stamping => {
                // Grumbo stamps a tally sheet, one thump per beat.
                let beat = if self.alt(0.2) { 0.0 } else { 1.0 };
                look.props.push(prop(rows(trick_art::HERO_PAD), 14.0, 0.0));
                look.props.push(prop(rows(trick_art::HERO_STAMP), 15.0, 4.0 + beat * 3.0));
            }
            TypingStyle::Spool => {
                // Wisp-a-ling winds glowing keystrokes onto a spool.
                look.props.push(prop(rows(trick_art::HERO_THREAD_SPOOL), 15.0, 0.0));
                for i in 0..6 {
                    let lift = 3.0 - (2.0 * (t * 7.0 + i as f64).sin()).round();
                    look.props.push(prop(rows(trick_art::HERO_SPARK), 12.0 + i as f64, lift));
                }
            }
            TypingStyle::SpellNotes => {
                // Hexley copies spell notes; the writing floats off the page.
                let beat = if self.alt(0.2) { 0.0 } else { 1.0 };
                look.props.push(prop(rows(trick_art::HERO_BOOK), 14.0, 0.0));
                look.props.push(prop(rows(trick_art::HERO_WAND), 14.0, 5.0 + beat));
                look.props.push(prop(rows(trick_art::HERO_SPARK), 18.0, (7.0 + 5.0 * (t % 2.0)).round()));
            }
            TypingStyle::Pebbles => {
                // Mossback sorts pebbles into a little row.
                let beat = if self.alt(0.2) { 0.0 } else { 1.0 };
                look.props.push(prop(rows(trick_art::HERO_PEBBLES), 14.0, 0.0));
                look.props.push(prop(rows(trick_art::HERO_ROCK), 14.0 + 5.0 * beat, 3.0 + 2.0 * beat));
            }
            TypingStyle::Checklist => {
                // Sir Clank punches holes in a checklist.
                let beat = if self.alt(0.2) { 0.0 } else { 1.0 };
                look.props.push(prop(rows(trick_art::HERO_PAD), 14.0, 0.0));
                look.props.push(prop(rows(trick_art::HERO_PAW), 15.0, 6.0 - 2.0 * beat));
            }
            TypingStyle::Mortar => {
                // Lunette pounds rice cakes in time with the keys.
                let beat = if self.alt(0.2) { 0.0 } else { 1.0 };
                look.props.push(prop(rows(trick_art::LUNETTE_MORTAR), 14.0, 0.0));
                look.props.push(prop(rows(trick_art::LUNETTE_MALLET), 14.0, 3.0 + beat * 4.0));
            }
            TypingStyle::Orbs => {
                // Orbis keeps an orb bobbing in front of him, a different one every so often.
                let kind = ((self.now - self.mode_start) / 0.9) as usize % 3;
                let bob = if self.alt(0.2) { 0.0 } else { 2.0 };
                let (x, y) = self.witcher_hand(0.0);
                look.free.push(self.orb_sprite(kind, x, y - 8.0 + bob, false));
            }
            TypingStyle::Moonlight => {
                // Moona waits on her panther while a little moon glows in front of it, twinkling as you type.
                let beat = if self.alt(0.2) { 0.0 } else { 1.0 };
                look.props.push(prop(rows(trick_art::MOONA_MOON), 13.0, 15.0 + beat));
                if beat > 0.0 {
                    look.props.push(prop(rows(trick_art::MOONA_TWINKLE), 12.0, 21.0));
                }
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

            // Hero pets. Timings follow prototypes/hero-pets.html; every trick lasts 8 s.
            _ => return self.hero_trick_look(tr, look),
        }
        look
    }

    fn hero_trick_look(&self, tr: Trick, mut look: Look) -> Look {
        use trick_art::*;
        let t = self.now - self.mode_start;
        let d = self.mode_until - self.mode_start;
        let eyes = self.blink_eyes();
        let p = self.pet;
        let beat = if (t * 5.0) as i64 % 2 == 1 { 1.0 } else { 0.0 }; // flips every 0.2 s

        match tr {
            Trick::GrumboSpin => {
                // Spins through four facings, topples over with stars circling, then gets up.
                if t < 3.0 {
                    look.frame = rotated(p.frame(eyes, Pose::Stand(0)), (t * 5.0) as usize % 4);
                    look.props = vec![prop(rows(HERO_AXE), 15.0, 2.0)];
                } else if t < 6.0 {
                    look.frame = rotated(p.frame(Eyes::Closed, Pose::Stand(0)), 1);
                    look.y_offset = 2.0;
                    look.free = vec![self.placed(HERO_STAR, 6.0 + (6.0 * (t * 5.0).sin()).round(), 16.0)];
                } else {
                    look.frame = p.frame(Eyes::Happy, Pose::Stand(0));
                }
            }
            Trick::GrumboHelmet => {
                // The helmet drops over his eyes, he shuffles about blind, then pushes it back up.
                look.frame = p.frame(if t >= 6.0 { Eyes::Happy } else { eyes }, Pose::Stand(0));
                if t > 1.0 && t < 6.0 {
                    look.x_offset = beat;
                    look.props = vec![prop(rows(HERO_HELMET), 1.0, 3.0), prop(rows(HERO_AXE), 15.0, 0.0)];
                }
            }
            Trick::GrumboChop => {
                // Lifts the axe, strikes the log, and the axe bounces straight back up. The log is fine.
                look.frame = p.frame(eyes, Pose::Stand(0));
                let lift = if t < 2.0 {
                    10.0
                } else if t < 4.0 {
                    2.0
                } else if t < 6.0 {
                    (2.0 + 10.0 * ((t - 4.0) / 2.0 * PI).sin()).round()
                } else {
                    3.0
                };
                look.props = vec![prop(rows(HERO_LOG), 17.0, 0.0), prop(rows(HERO_AXE), 15.0, lift)];
                if t > 4.0 && t < 6.0 {
                    look.y_offset = -beat;
                }
            }

            Trick::WispThread => {
                // A glowing thread stretches out to the pointer while the wisp's glow pulses.
                look.frame = p.frame(eyes, Pose::Stand(0));
                look.palette.set('w', Some(if beat > 0.0 { hex(0xfff7d8) } else { hex(0xe9b878) }));
                let reach = if t < 2.0 { t / 2.0 } else if t > d - 2.0 { (d - t).max(0.0) / 2.0 } else { 1.0 };
                let target = self.thread_target(t);
                let from = pt(self.center_x(14.0), 5.0 * PIXEL);
                let end = pt(from.x + (target.x - from.x) * reach, from.y + (target.y - from.y) * reach);
                let steps = 2.max(((end.x - from.x).hypot(end.y - from.y) / 5.0) as usize);
                look.free = (0..=steps)
                    .map(|i| {
                        let q = i as f64 / steps as f64;
                        free(rows(HERO_SPARK), from.x + (end.x - from.x) * q - 3.0, from.y + (end.y - from.y) * q - 3.0, 2.0)
                    })
                    .collect();
                look.free.push(free(rows(HERO_STAR), end.x - 5.0, end.y - 5.0, 2.0));
            }
            Trick::WispKnot => {
                // Winds a glowing loop, ends up with it knotted on its face, then shakes free.
                if t < 3.0 {
                    look.frame = p.frame(eyes, Pose::Stand(0));
                    look.free = vec![self.placed(HERO_KNOT, 16.0 - (t * 3.0).round(), 5.0)];
                } else if t < 6.0 {
                    look.frame = p.frame(Eyes::Closed, Pose::Stand(0));
                    look.x_offset = beat;
                    look.props = vec![prop(rows(HERO_KNOT), 7.0, 5.0)];
                } else {
                    look.frame = p.frame(Eyes::Happy, Pose::Stand(0));
                }
            }
            Trick::WispMoth => {
                // A moth circles the glow, lands on its head, and the wisp hops in surprise.
                look.frame = p.frame(if t > 7.0 { Eyes::Happy } else { eyes }, Pose::Stand(0));
                if t < 5.0 {
                    let dx = 8.0 + (14.0 * (t * 3.0).cos()).round();
                    let up = 16.0 - (4.0 * (t * 3.0).sin()).round();
                    look.free = vec![self.placed(HERO_MOTH, dx, up)];
                } else {
                    look.free = vec![self.placed(HERO_MOTH, 7.0, if t > 7.0 { 17.0 } else { 14.0 })];
                }
                if t > 7.0 {
                    look.y_offset = 3.0;
                }
            }

            Trick::HexFrog | Trick::HexSheep => {
                // Waves the wand, vanishes in a puff, bounces about as a frog or sheep, then puffs back.
                look.frame = p.frame(if t >= 7.0 { Eyes::Happy } else { eyes }, Pose::Stand(0));
                let hidden = [[b'.'; 16]; 16];
                if t < 2.0 {
                    look.props = vec![prop(rows(HERO_WAND), 14.0, 4.0 + beat)];
                } else if t < 3.0 || (6.0..7.0).contains(&t) {
                    look.frame = hidden;
                    look.free = vec![self.placed(HERO_PUFF, 4.0, 5.0)];
                } else if t < 6.0 {
                    look.frame = hidden;
                    let critter = if tr == Trick::HexFrog { HERO_FROG } else { HERO_SHEEP };
                    look.free = vec![self.placed(critter, 4.0, (3.0 * (t * 5.0).sin().abs()).round())];
                }
            }
            Trick::HexBook => {
                // A book floats up, snaps shut on his nose, then drops while he recoils.
                let biting = t > 3.0 && t < 5.0;
                look.frame = p.frame(if biting { Eyes::Closed } else { eyes }, Pose::Stand(0));
                let col = if t < 3.0 { 16.0 } else if biting { 14.0 } else { 17.0 }; // 14 while he recoils 2 px, so the book stays put
                let lift = if t < 3.0 {
                    (4.0 + t).round()
                } else if t < 5.0 {
                    6.0
                } else {
                    (6.0 - (t - 5.0) * 3.0).max(0.0).round()
                };
                look.props = vec![prop(rows(HERO_BOOK), col, lift)];
                if biting {
                    look.x_offset = -2.0;
                }
            }

            Trick::MossToss => {
                // Tosses a pebble in an arc, then naps standing up.
                if t < 3.0 {
                    look.frame = p.frame(eyes, Pose::Stand(0));
                    look.free = vec![self.placed(HERO_ROCK, 14.0 + t * 5.0, (16.0 * (t / 3.0 * PI).sin()).round())];
                } else {
                    look.frame = p.frame(Eyes::Closed, Pose::Stand(0));
                    look.free = vec![
                        self.placed(HERO_ROCK, 29.0, 0.0),
                        self.placed(HERO_Z, 10.0, 17.0 + (2.0 * (t % 2.0)).round()),
                    ];
                }
            }
            Trick::MossBloom => {
                // A flower pushes up through the moss while he sleeps, then folds away again.
                look.frame = p.frame(if t > 3.0 { Eyes::Happy } else { Eyes::Closed }, Pose::Stand(0));
                if t > 1.0 && t < 7.0 {
                    look.props = vec![prop(rows(HERO_FLOWER), 7.0, (14.0 + (t - 1.0).min(3.0)).round())];
                }
            }
            Trick::MossRain => {
                // A little drizzle of his own; the moss drinks it up and brightens, then a happy shake.
                if t < 6.0 {
                    look.frame = p.frame(eyes, Pose::Stand(0));
                    let fall1 = ((t * 9.0) % 8.0).round();
                    let fall2 = ((t * 9.0 + 3.0) % 8.0).round();
                    look.free = vec![self.placed(HERO_RAIN, 4.0, 19.0 - fall1), self.placed(HERO_RAIN, 10.0, 19.0 - fall2)];
                    if beat > 0.0 {
                        look.palette.set('m', Some(hex(0xa8bc77)));
                    }
                } else {
                    look.frame = p.frame(Eyes::Happy, Pose::Stand(0));
                    look.x_offset = beat;
                }
            }

            Trick::ClankWind => {
                // Steps slow to a stop (step_trick moves him), the key on his back turns, then off he goes.
                if t < 3.0 {
                    look.frame = p.frame(eyes, Pose::Stand((8.0 * (1.0 - (-t).exp())) as usize % 2));
                } else if t < 7.0 {
                    look.frame = p.frame(Eyes::Closed, Pose::Stand(0));
                } else {
                    look.frame = p.frame(Eyes::Happy, Pose::Stand((self.clock / 0.15) as usize % 2));
                }
                let turning = t > 5.0 && t < 7.0 && beat > 0.0;
                look.back = vec![self.placed(if turning { HERO_KEY_B } else { HERO_KEY_A }, -4.0, 4.0)];
            }
            Trick::ClankSpring => {
                // A spring pops out of his hatch, wobbles, and gets reeled back in.
                look.frame = p.frame(eyes, Pose::Stand(0));
                if t > 2.0 && t < 7.0 {
                    let out = ((if t < 5.0 { t - 2.0 } else { 7.0 - t }) * 2.0).round();
                    look.props = vec![prop(rows(HERO_SPRING), 12.0 + out + if t < 3.0 { 2.0 } else { 0.0 }, 3.0)];
                }
                if t > 2.0 && t < 3.0 {
                    look.x_offset = -2.0;
                }
            }
            Trick::ClankBell => {
                // An alarm bell rises out of his helmet and rattles; he jumps.
                let rattling = t > 3.0 && t < 6.0;
                look.frame = p.frame(eyes, Pose::Stand(0));
                if t < 7.0 {
                    let col = 7.0 + if rattling { beat } else { 0.0 };
                    look.props = vec![prop(rows(HERO_BELL), col, (14.0 + t.min(3.0)).round())];
                }
                if rattling {
                    look.y_offset = beat * 2.0;
                }
            }

            Trick::LunetteRice => {
                // Pounds rice in the mortar, then proudly holds up a rice cake.
                look.frame = p.frame(if t < 5.0 { eyes } else { Eyes::Happy }, Pose::Stand(0));
                look.props = vec![prop(rows(LUNETTE_MORTAR), 14.0, 0.0)];
                if t < 5.0 {
                    look.props.push(prop(rows(LUNETTE_MALLET), 14.0, 3.0 + ((t * 2.0) as i64 % 2) as f64 * 4.0));
                } else {
                    look.props.push(prop(rows(LUNETTE_CAKE), 14.0, 5.0));
                    if beat > 0.0 {
                        look.free = vec![self.placed(HERO_SPARK, 21.0, 10.0)];
                    }
                }
            }
            Trick::LunetteWish => {
                // Gazes at the stars, a shooting star crosses, she closes her eyes and wishes.
                let wish_eyes = if t >= 6.0 { Eyes::Happy } else if t >= 4.0 { Eyes::Closed } else { Eyes::Open };
                look.frame = p.frame(wish_eyes, Pose::Sit);
                look.free = vec![self.placed(HERO_STAR, -3.0, 23.0), self.placed(HERO_STAR, 22.0, 26.0)];
                if (2.0..4.0).contains(&t) {
                    look.free.push(self.placed(LUNETTE_COMET, -8.0 + (t - 2.0) * 22.0, (27.0 - (t - 2.0) * 4.5).round()));
                }
                if t >= 4.0 && beat > 0.0 {
                    look.free.push(self.placed(HERO_SPARK, 17.0, 17.0));
                    look.free.push(self.placed(HERO_SPARK, 5.0, 22.0));
                }
            }
            Trick::LunetteNap => {
                // Tucks her ears down and naps on a crescent-moon cushion.
                let napping = (1.0..7.0).contains(&t);
                let nap_eyes = if t < 1.0 { Eyes::Open } else if napping { Eyes::Closed } else { Eyes::Happy };
                let mut frame = p.frame(nap_eyes, Pose::Sit);
                if napping {
                    for (row, ears) in frame.iter_mut().zip(LUNETTE_TUCKED_EARS) {
                        row.copy_from_slice(ears.as_bytes());
                    }
                }
                look.frame = frame;
                look.y_offset = if napping { 1.0 } else { 2.0 }; // resting on the cushion
                look.back = vec![self.placed(LUNETTE_CUSHION, -3.0, 0.0)];
                if napping {
                    let drift = (2.0 * (t % 2.0)).round();
                    look.free = vec![self.placed(HERO_Z, 17.0, 18.0 + drift)];
                    if t > 3.0 {
                        look.free.push(self.placed(HERO_Z, 23.0, 22.0 + drift));
                    }
                }
            }
            Trick::MoonaPounce => {
                // Gathers, springs forward in an arc, lands with a puff of dust.
                if (POUNCE_UP..POUNCE_LAND).contains(&t) {
                    let q = (t - POUNCE_UP) / (POUNCE_LAND - POUNCE_UP);
                    look.frame = Picture::marker_index(pictures::MOONA_JUMP);
                    look.y_offset = (8.0 * 4.0 * q * (1.0 - q)).round();
                } else {
                    look.frame = p.frame(if t >= POUNCE_LAND + 0.4 { Eyes::Happy } else { eyes }, Pose::Sit);
                }
                if (POUNCE_LAND..POUNCE_LAND + 0.45).contains(&t) {
                    look.free = vec![self.placed(HERO_PUFF, -14.0, 0.0), self.placed(HERO_PUFF, 22.0, 0.0)];
                }
            }
            Trick::WitcherOrb => {
                // Holds out an orb, it bursts from his hand and flies off ahead, popping where it lands.
                let kind = (self.mode_start * 7.0) as usize % 3; // a different orb from throw to throw
                look.frame = if t < ORB_LAUNCH {
                    Picture::marker_index(pictures::WITCHER_THROW)
                } else if t < ORB_LAUNCH + 0.25 {
                    Picture::marker_index(pictures::WITCHER_BURST)
                } else if t < ORB_LAND {
                    Picture::marker_index(pictures::WITCHER_AFTER)
                } else {
                    p.frame(Eyes::Happy, Pose::Sit)
                };
                if (ORB_LAUNCH..ORB_LAND).contains(&t) {
                    for (lag, faint) in [(0.08, true), (0.04, true), (0.0, false)] {
                        let q = ((t - lag - ORB_LAUNCH) / (ORB_LAND - ORB_LAUNCH)).max(0.0);
                        let (x, y) = self.witcher_hand(q);
                        look.free.push(self.orb_sprite(kind, x, y, faint));
                    }
                }
                if (ORB_LAND..ORB_LAND + 0.4).contains(&t) {
                    let (x, y) = self.witcher_hand(1.0);
                    look.free.push(free(rows(HERO_PUFF), x - 16.0, (y - 14.0).max(0.0), PIXEL));
                    look.free.push(free(rows(MOONA_TWINKLE), x - 6.0, y - 6.0 + 20.0 * (t - ORB_LAND), PIXEL));
                }
            }
            Trick::WitcherWhirl => {
                // Draws the orbs round him faster and faster, then sets them off in a flash.
                look.frame = if t < ORB_WHIRL_START {
                    Picture::marker_index(pictures::WITCHER_CAST)
                } else if t < ORB_WHIRL_FLASH {
                    Picture::marker_index(pictures::WITCHER_WHIRL + self.step_of(0.12))
                } else if t < ORB_WHIRL_FLASH + 0.5 {
                    Picture::marker_index(pictures::WITCHER_BURST)
                } else if t < ORB_WHIRL_FLASH + 0.8 {
                    Picture::marker_index(pictures::WITCHER_AFTER)
                } else {
                    p.frame(Eyes::Happy, Pose::Sit)
                };
                if (ORB_WHIRL_FLASH..ORB_WHIRL_FLASH + 0.6).contains(&t) {
                    let c = self.pet_center_x();
                    let up = 20.0 * (t - ORB_WHIRL_FLASH);
                    for (dx, dy) in [(-44.0, 40.0), (40.0, 52.0), (-30.0, 70.0), (50.0, 24.0)] {
                        look.free.push(free(rows(MOONA_TWINKLE), c + dx, dy + up, PIXEL));
                    }
                }
            }
            Trick::WitcherWeigh => {
                // Turns to each orb in turn, orange, blue, pink, and round again.
                look.frame = if (0.2..3.4).contains(&t) {
                    Picture::marker_index(pictures::WITCHER_SIDE + ((t - 0.2) / 0.55) as usize % 3)
                } else {
                    p.frame(if t >= 3.4 { Eyes::Happy } else { eyes }, Pose::Sit)
                };
            }
            Trick::WitcherVanish => {
                // Fades into a ghost and away, then fades back in somewhere else.
                let fade = if t < 0.4 {
                    look.frame = Picture::marker_index(pictures::WITCHER_CAST);
                    1.0
                } else if t < VANISH_BACK + 0.8 {
                    look.frame = Picture::marker_index(pictures::WITCHER_GHOST);
                    if t < VANISH_GONE {
                        1.0 - ((t - 0.6) / (VANISH_GONE - 0.6)).clamp(0.0, 1.0)
                    } else if t < VANISH_BACK {
                        0.0
                    } else {
                        (t - VANISH_BACK) / 0.8
                    }
                } else {
                    look.frame = p.frame(Eyes::Happy, Pose::Sit);
                    1.0
                };
                if fade < 1.0 {
                    look.palette = look.palette.faded(fade);
                }
                // A wisp where he goes and where he comes back.
                for at in [VANISH_GONE - 0.3, VANISH_BACK] {
                    if (at..at + 0.5).contains(&t) {
                        let c = self.pet_center_x();
                        look.free.push(free(rows(MOONA_TWINKLE), c - 6.0, 40.0 + 30.0 * (t - at), PIXEL));
                    }
                }
            }
            Trick::MoonaGlaive => {
                // Throws her glaive like a boomerang: it spins out ahead of her, loops round and comes
                // back to her hand.
                if t < THROW_OUT {
                    look.frame = Picture::marker_index(pictures::MOONA_WEAPON);
                } else if t < THROW_BACK {
                    look.frame = Picture::marker_index(pictures::MOONA_BARE);
                    // A fading trail, then the glaive itself.
                    for (lag, faint) in [(0.07, true), (0.035, true), (0.0, false)] {
                        let q = ((t - lag - THROW_OUT) / (THROW_BACK - THROW_OUT)).max(0.0);
                        let (x, y) = self.glaive_at(q);
                        look.free.push(self.glaive_sprite(x, y, (t - lag) * 17.0, faint));
                    }
                } else {
                    look.frame = p.frame(if t > THROW_BACK + 0.15 { Eyes::Happy } else { eyes }, Pose::Sit);
                    if t < THROW_BACK + 0.25 {
                        let (x, y) = self.glaive_at(0.0);
                        look.free.push(free(rows(MOONA_TWINKLE), x - 6.0, y - 6.0, PIXEL));
                    }
                }
            }
            Trick::MoonaTriplicate => {
                // Two exact copies of her pop up on either side. All three send their glaives
                // whirling round themselves for a while, catch them, and the copies vanish.
                let spinning = (TRI_SPIN..TRI_CATCH).contains(&t);
                let frame = if spinning { pictures::MOONA_BARE } else { 2 }; // 2: standing still, glaive in hand
                look.frame = if spinning {
                    Picture::marker_index(pictures::MOONA_BARE)
                } else {
                    p.frame(if t >= TRI_CATCH { Eyes::Happy } else { eyes }, Pose::Sit)
                };
                let copies = self.copy_centers();
                if (TRI_APPEAR..TRI_VANISH).contains(&t) {
                    for (k, &x) in copies.iter().enumerate() {
                        let orbit = spinning.then(|| self.whirl(x, t, k as f64 + 1.0));
                        if let Some((behind, _)) = &orbit {
                            look.back.extend(behind.iter().map(|g| self.glaive_copy(g)));
                        }
                        look.back.push(self.picture_copy(frame, x));
                        if let Some((_, front)) = &orbit {
                            look.back.extend(front.iter().map(|g| self.glaive_copy(g)));
                        }
                    }
                }
                if spinning {
                    let (behind, front) = self.whirl(self.pet_center_x(), t, 0.0);
                    look.back.extend(behind.iter().map(|g| self.glaive_copy(g)));
                    look.free.extend(front.iter().map(|g| self.glaive_copy(g)));
                }
                // A puff where each copy appears and where it goes.
                for at in [TRI_APPEAR - 0.1, TRI_VANISH - 0.1] {
                    if (at..at + 0.4).contains(&t) {
                        for &x in &copies {
                            look.free.push(free(rows(HERO_PUFF), x - 16.0, 8.0, PIXEL));
                            look.free.push(free(rows(MOONA_TWINKLE), x - 6.0, 46.0 + 20.0 * (t - at), PIXEL));
                        }
                    }
                }
            }
            Trick::MoonaMoonlight => {
                // Raises her glaive and calls down the moon: a broad beam of light falls on her, and
                // narrow beams strike the ground all round, one after another.
                let raised = (MOON_UP..MOON_DOWN).contains(&t);
                look.frame = if raised {
                    Picture::marker_index(pictures::MOONA_STORM)
                } else {
                    p.frame(if t >= MOON_DOWN { Eyes::Happy } else { eyes }, Pose::Sit)
                };
                if (MOON_UP..MOON_DOWN + 0.2).contains(&t) {
                    let reach = ((t - MOON_UP) / 0.3).min(1.0);
                    let width = 28 + 2 * beat as usize;
                    look.back.push(self.moonbeam(self.pet_center_x(), width, reach, t > MOON_DOWN));
                }
                for i in 0..MOONBEAMS {
                    let age = t - (MOON_UP + 0.3 + i as f64 * 0.25);
                    if !(0.0..0.55).contains(&age) {
                        continue;
                    }
                    let x = self.moonbeam_x(i);
                    look.free.push(self.moonbeam(x, 6, (age / 0.12).min(1.0), age > 0.4));
                    if (0.12..0.4).contains(&age) {
                        look.free.push(free(rows(MOONA_TWINKLE), x - 6.0, 0.0, PIXEL));
                        look.free.push(free(rows(HERO_PUFF), x - 8.0, 0.0, PIXEL / 2.0));
                    }
                }
            }
            _ => {}
        }
        look
    }

    // MARK: Orbis's orbs

    /// Where a cast orb is (pen points, its centre), `q` of the way along its flight: from his hand,
    /// straight out ahead, dipping a little as it goes.
    fn witcher_hand(&self, q: f64) -> (f64, f64) {
        let px = self.pet.picture.as_ref().map_or(1.0, |p| p.px);
        let dir = if self.facing_left { -1.0 } else { 1.0 };
        let hand_x = self.pet_center_x() + dir * pictures::WITCHER_HAND.0 * px;
        let hand_y = pictures::WITCHER_HAND.1 * px;
        let room = if self.facing_left { hand_x - 14.0 } else { self.width - 14.0 - hand_x };
        let reach = room.clamp(30.0, 150.0);
        let q = q.clamp(0.0, 1.0);
        (hand_x + dir * reach * q, hand_y - 22.0 * q * q)
    }

    /// One of his orbs (0 orange, 1 blue, 2 pink) centred at (x, y); a see-through one for its trail.
    fn orb_sprite(&self, kind: usize, x: f64, y: f64, faint: bool) -> FreeSprite {
        let px = self.pet.picture.as_ref().map_or(PIXEL / 2.0, |p| p.px);
        let trail = ['S', 'T', 'U'][kind % 3];
        let grid: Rows = pictures::WITCHER_ORBS[kind % 3]
            .iter()
            .map(|r| r.chars().map(|ch| if faint && ch != '.' { trail } else { ch }).collect())
            .collect();
        let (w, h) = (grid[0].len() as f64 * px, grid.len() as f64 * px);
        FreeSprite { own_colors: true, ..free(grid, x - w / 2.0, y - h / 2.0, px) }
    }

    // MARK: Moona's glaive and moonbeams

    /// Where Moona's thrown glaive is (pen points, its centre), `q` of the way round its loop: out
    /// ahead of her, high on the way out and lower on the way back, then into her hand.
    fn glaive_at(&self, q: f64) -> (f64, f64) {
        let (anchor, px) = self.pet.picture.as_ref().map_or((0.0, 1.0), |p| (p.anchors[0], p.px));
        let dir = if self.facing_left { -1.0 } else { 1.0 };
        let hand_x = self.pet_center_x() + dir * (pictures::MOONA_HAND.0 - anchor) * px;
        let hand_y = pictures::MOONA_HAND.1 * px;
        let room = if self.facing_left { hand_x - 14.0 } else { self.width - 14.0 - hand_x };
        let rx = (room / 2.0).clamp(18.0, 70.0);
        let th = q.clamp(0.0, 1.0) * 2.0 * PI;
        let rise = if th < PI { 30.0 } else { 12.0 };
        (hand_x + dir * rx * (1.0 - th.cos()), hand_y + rise * th.sin())
    }

    /// The spinning glaive centred at (x, y), turned `angle` radians; a faint one for its trail.
    fn glaive_sprite(&self, x: f64, y: f64, angle: f64, faint: bool) -> FreeSprite {
        let px = self.pet.picture.as_ref().map_or(PIXEL / 2.0, |p| p.px);
        let grid = crescent(angle, 7.0, faint);
        let half = grid.len() as f64 * px / 2.0;
        FreeSprite { own_colors: true, ..free(grid, x - half, y - half, px) }
    }

    /// A shaft of moonlight `width` cells wide centred on x, reaching `reach` of the way down from
    /// the top of the pen; faint as it fades.
    fn moonbeam(&self, x: f64, width: usize, reach: f64, faint: bool) -> FreeSprite {
        let px = PIXEL / 2.0;
        let rows_n = ((self.height / px).ceil() * reach).round().max(1.0) as usize;
        // Brightest down the middle, softer to the sides, and fading out toward the top.
        let levels = ['.', 'R', 'S', 'T', 'U'];
        let mid = (width as f64 - 1.0) / 2.0;
        let across: Vec<usize> = (0..width)
            .map(|c| {
                let d = (c as f64 - mid).abs() / (width as f64 / 2.0);
                let level = if d > 0.75 { 1 } else if d > 0.5 { 2 } else if d > 0.2 { 3 } else { 4 };
                if faint { level - 1 } else { level }
            })
            .collect();
        let grid: Rows = (0..rows_n)
            .map(|r| {
                let dim = if r * 10 < rows_n * 3 { 1 } else { 0 };
                across.iter().map(|&l| levels[l.saturating_sub(dim)]).collect()
            })
            .collect();
        let left = x - width as f64 * px / 2.0;
        FreeSprite { own_colors: true, ..free(grid, left, self.height - rows_n as f64 * px, px) }
    }

    /// Where Moona's two copies stand when she triplicates: either side of her, inside the pen.
    fn copy_centers(&self) -> [f64; 2] {
        let c = self.pet_center_x();
        let w = self.pet.size().0;
        let (lo, hi) = (w * 0.45, (self.width - w * 0.45).max(w * 0.45));
        [(c - w * 0.8).clamp(lo, hi), (c + w * 0.8).clamp(lo, hi)]
    }

    /// One of her picture frames drawn as a separate figure centred on x (a copy of her).
    fn picture_copy(&self, frame: usize, center_x: f64) -> FreeSprite {
        let Some(pic) = self.pet.picture.as_ref() else { return free(vec![], 0.0, 0.0, PIXEL) };
        let grid = &pic.frames[frame];
        let w = grid[0].len() as f64;
        let anchor = if self.facing_left { w - pic.anchors[frame] } else { pic.anchors[frame] };
        let rows: Rows = grid
            .iter()
            .map(|r| {
                let s: String = r.iter().map(|&b| b as char).collect();
                if self.facing_left { s.chars().rev().collect() } else { s }
            })
            .collect();
        FreeSprite { own_colors: true, ..free(rows, center_x - anchor * pic.px, 0.0, pic.px) }
    }

    /// A glaive whirling round a figure centred on x, on a flat ring at chest height: its trail and
    /// itself, split into the parts behind the figure and in front. `phase` keeps the three apart.
    fn whirl(&self, x: f64, t: f64, phase: f64) -> (Vec<(f64, f64, f64, bool)>, Vec<(f64, f64, f64, bool)>) {
        let rx = self.pet.size().0 * 0.55;
        let (mut behind, mut front) = (vec![], vec![]);
        for (lag, faint) in [(0.06, true), (0.03, true), (0.0, false)] {
            let tt = t - lag - TRI_SPIN;
            let a = phase * 2.1 + tt * 7.0 + 2.0 * tt * tt.min(1.0); // speeds up as it gets going
            let g = (x + rx * a.cos(), 50.0 + 12.0 * a.sin(), (t - lag) * 17.0, faint);
            if a.sin() > 0.0 { behind.push(g) } else { front.push(g) }
        }
        (behind, front)
    }

    fn glaive_copy(&self, &(x, y, angle, faint): &(f64, f64, f64, bool)) -> FreeSprite {
        self.glaive_sprite(x, y, angle, faint)
    }

    /// Where the i-th narrow moonbeam strikes: spread over the pen, clear of Moona herself.
    fn moonbeam_x(&self, i: usize) -> f64 {
        let (lo, hi) = (10.0, (self.width - 10.0).max(11.0));
        let f = (0.37 + i as f64 * 0.618_034) % 1.0;
        let mut x = lo + f * (hi - lo);
        let c = self.pet_center_x();
        if (x - c).abs() < 48.0 {
            let side = if i % 2 == 0 { 1.0 } else { -1.0 };
            x = c + side * (52.0 + 9.0 * (i % 3) as f64);
            if x < lo || x > hi {
                x = c - side * (52.0 + 9.0 * (i % 3) as f64);
            }
        }
        x.clamp(lo, hi)
    }
}

/// Moona's glaive leaves her hand, and is back.
const THROW_OUT: f64 = 0.22;
const THROW_BACK: f64 = 1.75;
/// Orbis: an orb leaves his hand and lands; the whirl starts and flashes; he fades out, moves
/// while he's gone, and fades back in.
const ORB_LAUNCH: f64 = 0.35;
const ORB_LAND: f64 = 1.2;
const ORB_WHIRL_START: f64 = 0.3;
const ORB_WHIRL_FLASH: f64 = 2.2;
const VANISH_GONE: f64 = 1.3;
const VANISH_MOVE: f64 = 1.7;
const VANISH_BACK: f64 = 2.3;
/// Triplicate: the copies appear, the glaives start whirling, are caught, and the copies go.
const TRI_APPEAR: f64 = 0.45;
const TRI_SPIN: f64 = 0.8;
const TRI_CATCH: f64 = 4.0;
const TRI_VANISH: f64 = 4.55;
/// She raises her glaive to the moon, and lowers it; how many beams strike the ground.
const MOON_UP: f64 = 0.3;
const MOON_DOWN: f64 = 3.8;
const MOONBEAMS: usize = 11;

/// A crescent blade of radius `r` cells, its hollow facing `angle` radians (counter-clockwise from
/// the right): a bright core inside a glowing edge, with a soft halo. Faint: all trail colour.
fn crescent(angle: f64, r: f64, faint: bool) -> Rows {
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
                        if faint { 'P' } else if edge >= 1.0 { 'N' } else { 'G' }
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


/// When Moona leaves the ground in her pounce, and when she lands.
const POUNCE_UP: f64 = 0.3;
const POUNCE_LAND: f64 = 1.25;

