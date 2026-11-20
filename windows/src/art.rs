//! Pixel art and pet designs, ported from main.swift. Grids are 16×16, facing right, row 0 at the top.

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
        Color { a: (a.clamp(0.0, 1.0) * 255.0).round() as u8, ..self }
    }
}

pub fn rgb(r: f64, g: f64, b: f64) -> Color {
    let c = |v: f64| (v * 255.0).round() as u8;
    Color::new(c(r), c(g), c(b), 255)
}

/// Colours for grid characters (ASCII only).
#[derive(Clone, Copy)]
pub struct Palette([Option<Color>; 128]);

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

/// What a pet does now and then to pass the time (sitting down with it for a while).
#[derive(Clone, Copy, PartialEq)]
pub enum Pastime {
    Book,    // reads (Mochi, Puddle)
    Yoyo,    // plays with a yo-yo
    Knit,    // knits a scarf
    Flower,  // sits by a flower while a butterfly flutters round it
    Juggle,  // juggles three balls
    Bubbles, // blows bubbles
}

/// The hero pets' pastimes.
fn hero_pastime(id: &str) -> Option<Pastime> {
    Some(match id {
        "grumbo" | "wispaling" => Pastime::Juggle,
        "mossback" | "samurai" => Pastime::Flower,
        "sirclank" | "ninja" => Pastime::Yoyo,
        "lunette" => Pastime::Knit,
        _ => Pastime::Bubbles, // Hexley
    })
}

#[derive(Clone, Copy, PartialEq)]
pub enum TypingStyle {
    Laptop,
    Cooking,
    Swimming,
    ReadingGlasses,
    Digging,
    // hero pets
    Stamping,
    Spool,
    SpellNotes,
    Pebbles,
    Checklist,
    Mortar,
    Moonlight,
    Orbs,
    Scroll,
    Polish,
}

pub type Frame = [[u8; 16]; 16];

/// A pet design on the 16×16 grid, facing right. Sprite column 14 is the front outline,
/// where props are held; eyes are drawn on rows 6–7 at `eye_cols`.
pub struct Pet {
    pub id: &'static str,
    pub name: &'static str,
    pub palette: Palette,
    pub body: [&'static str; 13], // eyes left out
    pub feet: [&'static str; 2],  // two walking rows
    pub sit_base: Option<&'static str>, // a pet without legs keeps this row under it when it sits
    pub paw: u8,
    pub eye_cols: [usize; 2],
    pub tricks: &'static [Trick],
    pub typing: TypingStyle, // what the pet does while you type
    pub sweeps: bool,
    pub reads_with_glasses: bool,
    pub premium: bool, // needs an unlock code
    pub wind_key: bool, // Sir Clank's wind-up key, always drawn behind him
    pub floats: bool,   // bobs up and down while walking instead of stepping (Wisp-a-ling)
    pub picture: Option<Picture>, // drawn from a detailed picture instead of `body` (Moona)
    pub pastime: Option<Pastime>, // what it does now and then to pass the time
    pub trick_gap: (f64, f64), // seconds between the tricks it does on its own
    pub favorite_trick: Option<Trick>, // picked for about half the tricks it does on its own
    pub turn_trick: Option<Trick>, // done each time it turns round to walk the other way
    pub wake_trick: Option<Trick>, // done each time it wakes up
    pub drinks: bool, // drinks when reminded about water (otherwise just waits with you)
    pub click_trick: Option<Trick>,       // what a click does instead of petting
    pub right_click_trick: Option<Trick>, // what a right-click does instead of opening the menu
}

impl Pet {
    /// "Ember the fox" → "Ember"; "Sir Clank" stays whole.
    pub fn short_name(&self) -> &'static str {
        self.name.split(" the ").next().unwrap_or(self.name)
    }

    /// Width and height of the drawn pet in points: the 16×16 sprite box, or a picture pet's picture.
    pub fn size(&self) -> (f64, f64) {
        match &self.picture {
            Some(p) => (p.width as f64 * p.px, p.height as f64 * p.px),
            None => (PET_SIZE, PET_SIZE),
        }
    }

    /// Height in points of the pet's tallest frame (a picture pet's big moves reach higher).
    pub fn tallest(&self) -> f64 {
        match &self.picture {
            Some(p) => p.frames.iter().map(|f| f.len()).max().unwrap_or(0) as f64 * p.px,
            None => PET_SIZE,
        }
    }

    pub fn rows(&self, eyes: Eyes) -> [[u8; 16]; 13] {
        let mut grid = [[b'.'; 16]; 13];
        for (r, row) in self.body.iter().enumerate() {
            grid[r].copy_from_slice(&row.as_bytes()[..16]);
        }
        for &c in &self.eye_cols {
            match eyes {
                Eyes::Open => {
                    grid[6][c] = b'k';
                    grid[7][c] = b'k';
                }
                Eyes::Closed => {
                    grid[7][c - 1] = b'k';
                    grid[7][c] = b'k';
                }
                Eyes::Happy => {
                    grid[6][c] = b'k';
                    grid[7][c - 1] = b'k';
                    grid[7][c + 1] = b'k';
                }
            }
        }
        grid
    }

    pub fn frame(&self, eyes: Eyes, pose: Pose) -> Frame {
        if self.picture.is_some() {
            return Picture::marker(eyes, pose);
        }
        let rows = self.rows(eyes);
        let mut out = [[b'.'; 16]; 16];
        let mut n = 0;
        for r in rows {
            out[n] = r;
            n += 1;
        }
        let mut drop = 0;
        match pose {
            Pose::Stand(step) => out[n].copy_from_slice(self.feet[step % 2].as_bytes()),
            Pose::Sit => match self.sit_base {
                Some(base) => out[n].copy_from_slice(base.as_bytes()),
                None => drop = 1,
            },
        }
        if drop == 1 {
            // sitting lowers the body by one row
            for r in (1..16).rev() {
                out[r] = out[r - 1];
            }
            out[0] = [b'.'; 16];
        }
        out
    }
}

pub fn pets() -> Vec<Pet> {
    vec![
        Pet {
            id: "mochi",
            name: "Mochi",
            palette: Palette::new(&[
                ('k', rgb(0.20, 0.16, 0.24)),
                ('w', rgb(0.99, 0.95, 0.89)),
                ('s', rgb(0.89, 0.82, 0.76)),
                ('p', rgb(1.00, 0.62, 0.70)),
            ]),
            body: [
                "................", "...kk......kk...", "..kppk....kppk..", "..kpwkkkkkkwpk..",
                ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.",
                ".kwwwwwwpwwwwpk.", ".kswwwwwwwwwwwk.", ".ksswwwwwwwwwwk.", "..ksssswwwwwwk..",
                "...kkkkkkkkkk...",
            ],
            feet: ["..kwk.....kwk...", "....kwk.kwk....."],
            sit_base: None,
            paw: b'w',
            eye_cols: [9, 12],
            tricks: &[Trick::Yarn, Trick::BoxNap, Trick::Laser, Trick::Fishing],
            typing: TypingStyle::Laptop,
            sweeps: true,
            reads_with_glasses: false,
            premium: false,
            wind_key: false,
            floats: false,
            picture: None,
            pastime: Some(Pastime::Book),
            trick_gap: crate::pet::TRICK_GAP,
            favorite_trick: None,
            turn_trick: None,
            wake_trick: None,
            drinks: true,
            click_trick: None,
            right_click_trick: None,
        },
        Pet {
            id: "ember",
            name: "Ember the fox",
            palette: Palette::new(&[
                ('k', rgb(0.23, 0.14, 0.13)),
                ('w', rgb(0.95, 0.53, 0.23)),
                ('s', rgb(0.83, 0.42, 0.16)),
                ('c', rgb(1.00, 0.95, 0.86)),
            ]),
            body: [
                ".....k.....k....", "....kwk...kwk...", "....kwwk.kwwk...", "...kwwwkkkwwwk..",
                "...kwwwwwwwwwwk.", "...kwwwwwwwwwwk.", ".k.kwwwwwwwwwwk.", "kckkwwwwwwwwwwk.",
                "kwkkwwwwccccckk.", "kwwkswwwcccccck.", "kwwksswwwccccck.", ".kkkssswwwwwwk..",
                "....kkkkkkkkk...",
            ],
            feet: ["....kwk...kwk...", ".....kwk.kwk...."],
            sit_base: None,
            paw: b'w',
            eye_cols: [9, 12],
            tricks: &[Trick::Firework, Trick::Flashlight, Trick::FlipEgg, Trick::Campfire],
            typing: TypingStyle::Cooking,
            sweeps: false,
            reads_with_glasses: false,
            premium: false,
            wind_key: false,
            floats: false,
            picture: None,
            pastime: Some(Pastime::Yoyo),
            trick_gap: crate::pet::TRICK_GAP,
            favorite_trick: None,
            turn_trick: None,
            wake_trick: None,
            drinks: true,
            click_trick: None,
            right_click_trick: None,
        },
        Pet {
            id: "pip",
            name: "Pip the penguin",
            palette: Palette::new(&[
                ('k', rgb(0.12, 0.13, 0.20)),
                ('b', rgb(0.24, 0.29, 0.43)),
                ('w', rgb(1.00, 1.00, 1.00)),
                ('o', rgb(1.00, 0.65, 0.24)),
                ('p', rgb(1.00, 0.62, 0.70)),
            ]),
            body: [
                "................", "................", ".....kkkkkk.....", "...kkbbbbbbkk...",
                "..kbbbbbbwwwwk..", ".kbbbbbbwwwwwwk.", ".kbbbbbwwwwwwwk.", ".kbbbbbwwwwwwwko",
                ".kbbbbbwwwwwwpk.", ".kbbkbbwwwwwwwk.", ".kbbbkbwwwwwwwk.", "..kbbbkwwwwwwk..",
                "...kkkkkkkkkk...",
            ],
            feet: ["...kook..kook...", "....kook.kook..."],
            sit_base: None,
            paw: b'w',
            eye_cols: [9, 12],
            tricks: &[Trick::HideInWater, Trick::SpitWater, Trick::Swim, Trick::Bubbles],
            typing: TypingStyle::Swimming,
            sweeps: false,
            reads_with_glasses: false,
            premium: true,
            wind_key: false,
            floats: false,
            picture: None,
            pastime: Some(Pastime::Knit),
            trick_gap: crate::pet::TRICK_GAP,
            favorite_trick: None,
            turn_trick: None,
            wake_trick: None,
            drinks: true,
            click_trick: None,
            right_click_trick: None,
        },
        Pet {
            id: "puddle",
            name: "Puddle the slime",
            palette: Palette::new(&[
                ('k', rgb(0.14, 0.31, 0.42)),
                ('c', rgb(0.56, 0.85, 0.95)),
                ('H', rgb(1.00, 1.00, 1.00)),
                ('d', rgb(0.39, 0.71, 0.86)),
                ('p', rgb(1.00, 0.62, 0.70)),
            ]),
            body: [
                "................", "................", "................", ".....kkkkkk.....",
                "...kkcccccckk...", "..kcHcccccccck..", ".kcHcccccccccck.", ".kcccccccccccck.",
                ".kccccccpccccpk.", ".kcccccccccccck.", ".kdccccccccccck.", ".kddcccccccccck.",
                ".kddddcccccccck.",
            ],
            feet: ["..kkkkkkkkkkkk..", ".kkkkkkkkkkkkkk."],
            sit_base: Some("..kkkkkkkkkkkk.."),
            paw: b'c',
            eye_cols: [9, 12],
            tricks: &[Trick::IceCream, Trick::Twin, Trick::ColorShift, Trick::PopFood],
            typing: TypingStyle::ReadingGlasses,
            sweeps: false,
            reads_with_glasses: true,
            premium: true,
            wind_key: false,
            floats: false,
            picture: None,
            pastime: Some(Pastime::Book),
            trick_gap: crate::pet::TRICK_GAP,
            favorite_trick: None,
            turn_trick: None,
            wake_trick: None,
            drinks: true,
            click_trick: None,
            right_click_trick: None,
        },
        Pet {
            id: "sprout",
            name: "Sprout the seedling",
            palette: Palette::new(&[
                ('k', rgb(0.14, 0.25, 0.16)),
                ('w', rgb(0.71, 0.88, 0.54)),
                ('s', rgb(0.55, 0.76, 0.40)),
                ('L', rgb(0.36, 0.70, 0.32)),
                ('S', rgb(0.25, 0.54, 0.23)),
                ('p', rgb(1.00, 0.62, 0.70)),
            ]),
            body: [
                "..kkk...kkk.....", ".kLLLk.kLLLk....", "..kLLLkLLLk.....", "..kkkkSkkkkkkk..",
                ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.",
                ".kwwwwwwpwwwwpk.", ".kswwwwwwwwwwwk.", ".ksswwwwwwwwwwk.", "..ksssswwwwwwk..",
                "...kkkkkkkkkk...",
            ],
            feet: ["..kwk.....kwk...", "....kwk.kwk....."],
            sit_base: None,
            paw: b'w',
            eye_cols: [9, 12],
            tricks: &[Trick::GiveFlower, Trick::Shower, Trick::PlantSelf, Trick::GrowUp, Trick::ChaseButterfly],
            typing: TypingStyle::Digging,
            sweeps: false,
            reads_with_glasses: false,
            premium: true,
            wind_key: false,
            floats: false,
            picture: None,
            pastime: Some(Pastime::Flower),
            trick_gap: crate::pet::TRICK_GAP,
            favorite_trick: None,
            turn_trick: None,
            wake_trick: None,
            drinks: true,
            click_trick: None,
            right_click_trick: None,
        },
        // Tiny heroes (designed in prototypes/hero-pets.html)
        hero(
            "grumbo",
            "Grumbo the warrior",
            &[('k', hex(0x33293d)), ('w', hex(0xe9bd8e)), ('s', hex(0xba886c)), ('h', hex(0x96909e)), ('H', hex(0xc4bfca)), ('v', hex(0x827565))],
            [
                "................", "....kkkkkkkk....", "...khhhhhhhhkk..", "..khhHhhhhhhhhk.",
                "..khhhhhhhhhhhk.", ".kkkkkkkkkkkkkk.", ".kswwwwwwwwwwwk.", ".kswwwwwwwwwwwk.",
                ".kswwwwwwswwwsk.", ".kvvwwwwwwwwvvk.", ".kvvvvvvvvvvvvk.", "..kvvvvHHvvvvk..",
                "...kkkkkkkkkk...",
            ],
            ["...kvk....kvk...", ".....kvkkvk....."],
            None,
            &[Trick::GrumboSpin, Trick::GrumboHelmet, Trick::GrumboChop],
            TypingStyle::Stamping,
        ),
        Pet {
            floats: true,
            ..hero(
                "wispaling",
                "Wisp-a-ling the spirit",
                &[('k', hex(0x33293d)), ('w', hex(0xf7edb0)), ('s', hex(0xe9b878)), ('h', hex(0xfff7d8)), ('p', hex(0xe7a98d))],
                [
                    ".....kk.........", "....kwhk........", "...kwwk.........", "...kwwkkkkkk....",
                    "..kwwwwwwwhhkk..", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kswwwwwwwwwwwk.",
                    "..kswwwwpwwwwpk.", "...kswwwwwwwwwk.", "..kwswwwwwwwwk..", ".kwkkswwwwwwk...",
                    "..k..kkkkkkk....",
                ],
                ["......kk..kk....", ".....kk..kk....."],
                Some("......kk..kk...."),
                &[Trick::WispThread, Trick::WispKnot, Trick::WispMoth],
                TypingStyle::Spool,
            )
        },
        hero(
            "hexley",
            "Hexley the apprentice",
            &[('k', hex(0x33293d)), ('w', hex(0xedc6a7)), ('s', hex(0xbc947e)), ('h', hex(0x729d98)), ('H', hex(0xa8c5b5)), ('p', hex(0xdba4a7))],
            [
                "...kk...........", "..kHHk..........", "...khkkkkkk.....", "..khhhhhhhhkk...",
                ".khhHhhhhhhhhhk.", ".khhhwwwwwwwwwk.", ".khhhwwwwwwwwwk.", ".khhhwwwwwwwwwk.",
                ".khhhwwwpwwwwpk.", ".khhhhhwwwwhhhk.", ".khhhhhHHhhhhhk.", "..khhhhHHhhhhk..",
                "...kkkkkkkkkk...",
            ],
            ["...khk....khk...", ".....khkkhk....."],
            None,
            &[Trick::HexFrog, Trick::HexSheep, Trick::HexBook],
            TypingStyle::SpellNotes,
        ),
        hero(
            "mossback",
            "Mossback the pebble giant",
            &[('k', hex(0x33293d)), ('w', hex(0xc0b7a3)), ('s', hex(0x938b7f)), ('m', hex(0x82965c)), ('M', hex(0xb8c68a)), ('b', hex(0x796b57))],
            [
                "................", "......kkk.......", "...kkkMmMkkk....", "..kmMmmmMmmmk...",
                ".kmmmmmMmmmmmk..", ".kwwwwwwwwwwwwk.", ".kswwwwwwwwwwwk.", ".kswwwwwwwwwwwk.",
                ".kswwwwwswwwwsk.", ".kswbwwwwwwwwwk.", ".kssbwwwbwwwwwk.", "..ksswwwbwwwwk..",
                "...kkkkkkkkkk...",
            ],
            ["..kbk.....kbk...", "....kbk.kbk....."],
            None,
            &[Trick::MossToss, Trick::MossBloom, Trick::MossRain],
            TypingStyle::Pebbles,
        ),
        Pet {
            wind_key: true,
            ..hero(
                "sirclank",
                "Sir Clank",
                &[('k', hex(0x33293d)), ('w', hex(0xe2dbbf)), ('s', hex(0xb5ab96)), ('h', hex(0xbc8270)), ('H', hex(0xf7efd8)), ('y', hex(0xd7b365))],
                [
                    "................", ".....kkkkkk.....", "....kwHwwwwk....", "...kwwwwwwwwk...",
                    "..kwwwwwwwwwwk..", ".kwwkkkkkkkkwwk.", ".kwkwHHHHHHHHwk.", ".kwkwHHHHHHHHwk.",
                    ".kwwkkkkkkkkwwk.", ".kswwwhhhwwwwwk.", ".ksswwhHhwwwwwk.", "..kssswwwsswwk..",
                    "...kkkkkkkkkk...",
                ],
                ["...khk....khk...", ".....khkkhk....."],
                None,
                &[Trick::ClankWind, Trick::ClankSpring, Trick::ClankBell],
                TypingStyle::Checklist,
            )
        },
        hero(
            "lunette",
            "Lunette the moon rabbit",
            &[('k', hex(0x33293d)), ('w', hex(0xfff3df)), ('s', hex(0xd9c9e8)), ('p', hex(0xe8a5b7))],
            [
                "....kk...kk.....", "...kwpk.kpwk....", "...kwpk.kpwk....", "...kwpk.kpwk....",
                "...kwwkkkwwkk...", "..kwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.",
                "kwwkwwwwwwwwwpk.", "kwwkswwwwwwwwwk.", ".kkkswwwwwwwwwk.", "...ksssswwwwwk..",
                "....kkkkkkkkk...",
            ],
            ["....kwk....kwk..", "......kwkkwk...."],
            None,
            &[Trick::LunetteRice, Trick::LunetteWish, Trick::LunetteNap],
            TypingStyle::Mortar,
        ),
        hero(
            "ninja",
            "Kage the ninja",
            &[('k', hex(0x221d2e)), ('w', hex(0x3d4870)), ('N', hex(0x5a6894)), ('s', hex(0xf2c9a0)), ('r', hex(0xd8414a))],
            [
                "................", ".....kkkkkk.....", "....kwwwwwwk....", "rr.kwwwwwwwwk...",
                ".rrkrrrrrrrrrk..", "..kwwwwwwwwwwwk.", "..kwwwwwssssssk.", "..kwwwwwssssssk.",
                "..kwwwwwwwwwwwk.", "..kwNwwwwwwwwwk.", "..kwwNwwwwwwwk..", "...kwwwwwwwwk...",
                "....kkkkkkkk....",
            ],
            ["....kwk..kwk....", "......kwkkwk...."],
            None,
            &[Trick::NinjaSmoke, Trick::NinjaShuriken, Trick::NinjaHide, Trick::NinjaClones, Trick::NinjaFlip, Trick::NinjaMeditate],
            TypingStyle::Scroll,
        ),
        Pet {
            paw: b's',
            ..hero(
                "samurai",
                "Ronin the samurai",
                &[('k', hex(0x2a2230)), ('w', hex(0xc0392b)), ('s', hex(0xf0c8a0)), ('h', hex(0x34343f)), ('H', hex(0x5a5a6a)),
                  ('R', hex(0xe8b84a)), ('y', hex(0xf5d060))],
                [
                    "...y.......y....", "....y.....y.....", ".....yyyyy......", "....kkkkkkkk....",
                    "...khhhhhhhhk...", "..khhhhhhhhhhk..", ".khhkssssssssk..", ".khhkssssssssk..",
                    ".kHHkssssssssk..", ".kwwwkkkkkkkwk..", ".kwwwwRRwwwwwk..", ".kwRwwwwwwRwwk..",
                    "..kkkkkkkkkkkk..",
                ],
                ["...khk....khk...", ".....khkkhk....."],
                None,
                &[Trick::SamuraiDraw, Trick::SamuraiBow, Trick::SamuraiBamboo, Trick::SamuraiTea, Trick::SamuraiSpin, Trick::SamuraiFly],
                TypingStyle::Polish,
            )
        },
        // A moon knight riding a black panther, drawn from prototypes/moona.jpg.
        Pet {
            palette: Palette::new(&pictures::moona_palette()),
            paw: b'f',
            picture: Some(pictures::moona()),
            pastime: None,
            trick_gap: (30.0, 40.0), // she shows off often
            favorite_trick: Some(Trick::MoonaGlaive), // about half her tricks
            turn_trick: Some(Trick::MoonaGlaive),
            wake_trick: Some(Trick::MoonaMoonlight),
            drinks: false,
            click_trick: Some(Trick::MoonaGlaive),
            right_click_trick: Some(Trick::MoonaMoonlight),
            ..hero("moona", "Moona the moon knight", &[], [EMPTY_ROW; 13], [EMPTY_ROW; 2], None,
                   &[Trick::MoonaPounce, Trick::MoonaGlaive, Trick::MoonaTriplicate, Trick::MoonaMoonlight], TypingStyle::Moonlight)
        },
        // A sorcerer with three orbs, drawn from prototypes/witcher.jpg.
        Pet {
            palette: Palette::new(&pictures::witcher_palette()),
            paw: b'L',
            picture: Some(pictures::witcher()),
            pastime: None,
            drinks: false,
            click_trick: Some(Trick::WitcherOrb),
            right_click_trick: Some(Trick::WitcherWhirl),
            ..hero("witcher", "Orbis the witcher", &[], [EMPTY_ROW; 13], [EMPTY_ROW; 2], None,
                   &[Trick::WitcherOrb, Trick::WitcherWhirl, Trick::WitcherWeigh, Trick::WitcherVanish, Trick::WitcherSunStrike,
                     Trick::WitcherTornado, Trick::WitcherMeteor, Trick::WitcherMoonBlast, Trick::WitcherSpirits], TypingStyle::Orbs)
        },
    ]
}

const EMPTY_ROW: &str = "................";

/// A pet drawn from detailed pictures (any size) instead of the 16×16 grid, one per pose.
pub struct Picture {
    /// Indexed by `eyes * 3 + pose`: eyes open, closed, happy; poses standing (two walking
    /// steps) and sitting. Rows top to bottom; the bottom row stands on the ground.
    /// Frames past the first nine are a pet's own extra poses (Moona's leap, sleep, ...).
    pub frames: Vec<Vec<Vec<u8>>>,
    /// The column of each frame that stands at the pet's centre, so poses of different
    /// widths line up.
    pub anchors: Vec<f64>,
    pub width: usize,  // of the standing picture
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
    /// Orbs turn on a wheel behind it while it stands or walks (Orbis).
    pub orb_wheel: bool,
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
        Picture { frames, anchors, width, height, px, idle_sits: false, prop_shift: 0.0, walk: vec![], sleep: None, eat: None, orb_wheel: false }
    }

    /// A marker frame for any frame by index (the extra poses come after the first nine).
    pub fn marker_index(i: usize) -> Frame {
        let mut out = [[b'.'; 16]; 16];
        out[0][0] = PICTURE_MARK + i as u8;
        out
    }

    /// Which frame a marker frame asks for.
    pub fn index_for(&self, marker: &Frame) -> usize {
        let i = marker.iter().flatten().find(|&&b| b >= PICTURE_MARK).map_or(0, |&b| (b - PICTURE_MARK) as usize);
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
fn gait(grid: &[Vec<u8>], hip: usize, swing: f64, lift: usize) -> [Vec<Vec<u8>>; 4] {
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
                let near = [(y + 1, x), (y.wrapping_sub(1), x), (y, x + 1), (y, x.wrapping_sub(1))];
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
    let order: Vec<usize> = if legs.len() == 4 { vec![0, 3, 1, 2] } else { (0..legs.len()).collect() };
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

pub mod pictures {
    use super::Picture;

    /// Moona's extra poses, after the nine standard frames.
    pub const MOONA_JUMP: usize = 9;
    pub const MOONA_WEAPON: usize = 10;
    pub const MOONA_STORM: usize = 11;
    pub const MOONA_SLEEP: usize = 12;
    pub const MOONA_EAT: usize = 13;
    const MOONA_STEP_B: usize = 14;
    const MOONA_STEP_D: usize = 15;
    /// Empty-handed, while her glaive is in the air.
    pub const MOONA_BARE: usize = 16;
    /// Where she holds her glaive in the walking and empty-handed pictures: cells from the
    /// left, and up from the bottom.
    pub const MOONA_HAND: (f64, f64) = (27.0, 35.0);

    /// Moona riding her panther, from the poses in prototypes/moona.jpg (one art pixel ≈ 4.3 pixels
    /// there). Each pose has its own picture; walking lifts the legs of the walking picture.
    pub fn moona() -> Picture {
        let grid = |rows: &[&str]| -> Vec<Vec<u8>> { rows.iter().map(|r| r.as_bytes().to_vec()).collect() };
        let walk = grid(MOONA_WALK);
        let anchor = 36.5;
        let mut frames = Vec::new();
        for eyes in 0..3 {
            let mut face = walk.clone();
            if eyes == 1 {
                // The panther's glowing eye goes dark.
                for (r, c) in [(30, 66), (30, 67), (29, 69)] {
                    face[r][c] = b'f';
                }
            }
            let steps = super::gait(&face, 46, 4.0, 2);
            frames.push((steps[0].clone(), anchor));
            frames.push((steps[2].clone(), anchor));
            frames.push((face, anchor));
        }
        for (rows, anchor) in [(MOONA_JUMP_ROWS, 32.5), (MOONA_WEAPON_ROWS, 34.5),
                               (MOONA_STORM_ROWS, 36.5), (MOONA_SLEEP_ROWS, 28.0), (MOONA_EAT_ROWS, 27.5)] {
            frames.push((grid(rows), anchor));
        }
        // The in-between steps of the walk, where a pair of legs passes forward off the ground.
        let steps = super::gait(&walk, 46, 4.0, 2);
        frames.push((steps[1].clone(), anchor)); // MOONA_STEP_B
        frames.push((steps[3].clone(), anchor)); // MOONA_STEP_D
        frames.push((grid(MOONA_BARE_ROWS), anchor));
        assert_eq!(frames.len(), MOONA_BARE + 1, "Moona frame indices out of step");
        let mut pic = Picture::from_frames(frames, 1.6);
        pic.idle_sits = true;
        pic.walk = vec![0, MOONA_STEP_B, 1, MOONA_STEP_D];
        pic.prop_shift = 7.0;
        pic.sleep = Some(MOONA_SLEEP);
        pic.eat = Some(MOONA_EAT);
        pic
    }

    /// Her colours, plus see-through ones for glows, trails and moonbeams.
    pub fn moona_palette() -> Vec<(char, super::Color)> {
        let mut p: Vec<(char, super::Color)> = MOONA_COLORS.iter().map(|&(k, v)| (k, super::hex(v))).collect();
        for &(k, v, a) in MOONA_LIGHTS {
            p.push((k, super::hex(v).with_alpha(a)));
        }
        p
    }

    /// O glow round the glaive, P its fading trail; R, S, T, U moonlight from faintest to brightest.
    pub const MOONA_LIGHTS: &[(char, u32, f64)] = &[('O', 0x8ef6fd, 0.45), ('P', 0x6ee5f8, 0.3), ('R', 0x9fe3ff, 0.1),
                                                     ('S', 0xb8f0ff, 0.2), ('T', 0xd8faff, 0.36), ('U', 0xffffff, 0.62)];

    pub const MOONA_COLORS: &[(char, u32)] = &[
        ('a', 0x070417), ('b', 0x130d31), ('c', 0x241b48), ('d', 0x242235), ('e', 0x1f2c60), ('f', 0x35225c),
        ('g', 0x303a57), ('h', 0x37396e), ('i', 0x234a72), ('j', 0x48465c), ('k', 0x394a92), ('l', 0x49507e),
        ('m', 0x814447), ('n', 0x2e6a95), ('o', 0x4c5db5), ('p', 0x566493), ('q', 0x4367d2), ('r', 0x726780),
        ('s', 0x3d85b4), ('t', 0xbb5f72), ('u', 0x448fda), ('v', 0x6f81b5), ('w', 0x907d91), ('x', 0x5fa4c3),
        ('y', 0xac8ba2), ('z', 0x3eb8e3), ('A', 0xd9818f), ('B', 0x8ea3c9), ('C', 0x5bc4ed), ('D', 0xc29dad),
        ('E', 0x2cdef5), ('F', 0x76beeb), ('G', 0x47d8f6), ('H', 0xccbfc3), ('I', 0x6ee5f8), ('J', 0xaed3ee),
        ('K', 0x8ef6fd), ('L', 0xaffcfe), ('M', 0xd7f1fb), ('N', 0xc8fdfd),
    ];

    pub const MOONA_WALK: &[&str] = &[
        "......................................d.dd...............................",
        "......................................p..b...............................",
        ".....................................pb..dd..............................",
        "....................................inab.ac..............................",
        "....................................phecedf..............................",
        "....................................phehpee..............................",
        "................................j...ehcaiib..............................",
        "..............................jrBJwJgedwphb..............................",
        ".............................rjlNNNNrclywya..............................",
        "..............................MMNNJJvheyyya..............................",
        ".............................MBvBJhbbaahyw...............................",
        ".............................Mvvvcehcccbbaa..............................",
        ".............................MrgesCeabcbbbeb.............................",
        "...............................dxneeajceiheh.............................",
        "...............................bcefajilpbwahc............................",
        "...............................eefaaesnhpkpba............................",
        "...............................bcbagbeihekkb.............................",
        ".............................dbicb.aeeechbcbd............................",
        ".............................lbecd.dcccfhcabcdd..........................",
        "............................biebg...ccehhcabgfwb.........................",
        "............................gihc...accbehbdaacyw....bb..bad..............",
        "........................l...ghfa..dghhaecadgbaja..aaffcacfbacba..........",
        "........................g..behcd..dcfffenbdggcbabccbffafffbffccaa........",
        "......................p..hhchb...bjecbflxbbcfbacffcffbfffbffbcaccfcd.....",
        ".....................p..gohbdbjdgeeefbabgbclcacfffbfcaffcfffcafacfffbdca.",
        "....................gx..pocyywbpbneecbeccbahaaccbdffccffbfffcacfbffffbda.",
        "....................ki..ouhffbeeiiehceecfclcabcfcacfdffbfffaadbfffpffgba.",
        "....................xCngiLokkhkkhebbeeeebcpcaffffbgfcbfbfffaccfffffffffb.",
        "..................j.ixLuxvnCukkxbaabbeeeebjacfffffaffcccdfaafffffffBepBb.",
        ".................gh..xNxJqKiezubbcbbbceeeagaffffffbabbaaaabafffffbbefxec.",
        "................ghk.iINvkFiiieelfffbbbeecaaffffffffabaaffffffhexkcCFaffi.",
        "................xhsiCNJkKaleeeefhffcbabeebaffffffffafaffffbffefepfccfff..",
        "................ilinKMoFNaleeeeellfbcaceebaffffffffafccfffacffvffffffffb.",
        "................ingzLMoJffjeeeeeecjacfeeccacfffffffaffadbcbafffffffffdfcd",
        ".bcd............iIinLMnKhfblheeeeebcefaahgcbfffffffafffabcaaaffffcffffacd",
        ".ffc............bnCzNMINkhfagheeeeceefchhacccfffffbafffcaabbaadffaaffffc.",
        "dffa...........afhkxLMNLphfcaabdeececchhbaacbffffbbbfffffaaaa..adaaabbab.",
        "cffa..........affboqsNNLNBhbffbfbaebahhabacbcffcabffffffffffb....caBara..",
        "ffc..........bcfbahoFILNNMMBvJBhcccbababfbabfbbfffffffffefffa....dffga...",
        "ffc.........dffcaaffkqGCLMNMMBhfbcbabfbcfbacfffffffbfffiFefbd.....aba....",
        "dffd......dacfcadaffffeqhokkfcabegabbffbcaacffffffcafffexcbfd............",
        ".bffa...baffffa.dbfffffffcccccaaeebbbcfbbaaccfffffbafffffafc.............",
        ".afffcccfffcaa..dcfffffffcdcaabaeeaabaffbacfcbcffbabffffaafa.............",
        "..abfffffbbad...acfffffffaaaaaaaeeaaaacfaacffffcbaabcbaaaaba.............",
        "....aabbba......bbffffffcaaaaaaaceabaaaffabbffffaaabaaaaabfcd............",
        "...............abcffffffa..aaaaaaaaaadabfbafffffaaaaaaaaabff.............",
        ".............ddbcffffffbb..aaaaaaab...dabacfffffadd..aaccbffb............",
        "............daffffffffba..dabbaabdd.......afffffd.....aabbfffa...........",
        "............dffcffffcba...dbbbbbcb........dffcffd......abcbffd...........",
        "............dffcbbbaaa....abbbbca.........dbfcffd.......babffcd..........",
        "............cffaad........bbbba............affccd........aabbcb..........",
        "............cffba.........bbbba............acbffb.........aacffa.........",
        "............cffbd.........bbbbb............abfffb.........dafffa.........",
        "............fffbd..........abbba...........acfffd..........bfffa.........",
        "...........afffa............abbc............bfffc..........abcfa.........",
        "...........afffa............abbbda..........bcfffc........babcfd.........",
        "...........afffcda...........abbbbba........bffffbbd......abcda..........",
        "...........affffb............aabbbba.........cffbcca......bbbd...........",
        "............bbbbad...........................bcfacab......dddd...........",
        "..............................................aaaaaa.....................",
    ];
    const MOONA_JUMP_ROWS: &[&str] = &[
        ".........................................b..d.........................",
        "........................................dl..bc........................",
        "........................................vc..jc........................",
        ".......................................lhbeeacb.......................",
        "..................................Mr...jecelbhb.......................",
        "...............................MMMpMl..eebeckla.......................",
        "...............................jHMNMMHJvfcyeeb........................",
        "...............................MMMMMMMMvccDlHb........................",
        "..............................MvBBJJpppbhcyyyd........................",
        "..............................Mv.wvCxnhgbcyyw.........................",
        "..............................j..bpnnhhacdaad.........................",
        "................................dccckhbccbbap.........................",
        ".................................ahgbaeblpebp.........................",
        "................................bccaaeillcba..........................",
        "...............................bccbacexhlrlad........bbb..............",
        "...............................bcinbbchchkgba....acbacfcabba..........",
        "...............................aehkibbceaggddddabcfaffbffcbcaaa.......",
        "................................behhlbaaahecaccfbfdcffbffcfbccccd..a..",
        "...............................dbcaeebrwbhcacffbbfaffbfffdacbcfffcac..",
        "...............................iiegcafwDfcabfffbfffffdfffdaffaffffad..",
        "..............................nieefabbbalbacbfbaffffccffaadbfffvfhfb..",
        "............................beeeeebcbbaagabafffaffbfbfffacffffffefff..",
        "............................cehflbeeecfbeabcffffaffabacaafffffffifBnc.",
        "............................aaaabbeeeecjbaffffffbaaabbccafffifbicflfa.",
        "...........................gcccbbbceeeajadfffffffbcbffffbfefenfxlcfcx.",
        "...........................blfccbbbbeecgaaffffffffacfffcbffhfffffcffd.",
        "........................abiblfffcbbbccbgaaffffffffacafbaaffffffffffff.",
        "........................ineeclfffcbabeehacbfffffffaffaccacffffbffffaaa",
        "......................aaleeeealcfffbeeehacabffffffaffcabbabfffbdfffcba",
        ".....................afcleeeeebppceeeedjababcfffbccaffdaaaaafcaafffcfb",
        "....................ffffbheeeeeeaecfgchacbacbccbfffaffffffb..ccawava..",
        "...................fffffcbpeeeeebecebfgafbaffbcfffcaffezffb..dfaaaa...",
        "..................bffffffcagghlgeebbghaffbbfffffffaaffizeba...bffca...",
        "..aa.............bbfffffffffbaabebaaaafffafffccbbaabfffedaab...aaa....",
        ".cffa...........bbcfffffffbfffbabbacfbffbacfffffbaaafffaaafffa........",
        "bffc...........dfbffffffffabfcccabbbfccfbafbffbcfffcbcaaabfffcaa......",
        "bfc...........dccafffffffccabaeecbbbbfbbaaccbcffffffffaaabbcfffffa....",
        "bfc...........cfbbfffffffcffaaeeabbbaffaaaaabffffffffcbcabbccfbfffa...",
        "bfc..........bffabfffffffccfcdbeaaaaaafcaaaabbcffffffcfffaaabcccfffb..",
        "bffd.......afffbaffffffffbffaaacaaaaaadffcaa..aabfffaffffcdgdaabfffb..",
        ".bffbb..dbcffbb.affffffffdfaaaaaa...aabbbbab......dbacfffffd...abbbcd.",
        ".abfffabccffba..afffffffbbaaaaaba...................abffffffb...bbbca.",
        "...bfffffcca....ffffffffaaaaaaabj....................dffffffd...abaca.",
        "....dabbaaa....afffffffbaaaaaaba......................bfcffba....aab..",
        "..............bcffffffbbbbaaaab.......................abccbad.....a...",
        ".............acfffffffbabbbaaba........................agad.d.........",
        "............bfffffffbaabbbbbba..........................d.............",
        "...........bfffffcbaabbbbbbdb.........................................",
        "...........dffbbbaabbbbbaab...........................................",
        "...........ffcbb..abbbaa..............................................",
        "..........affca..abbbd................................................",
        "..........bffb...abbb.................................................",
        ".........dfffa...acba.................................................",
        ".........bfff...abcba.................................................",
        "........bfffb..babbb..................................................",
        ".......abfffa..bbbba..................................................",
        ".......affcfa..dbbb...................................................",
        ".......acacba...ba....................................................",
        "........aaab..........................................................",
    ];
    const MOONA_WEAPON_ROWS: &[&str] = &[
        ".........................................hhiiiiiiiiiiig................",
        ".......................................lkopppssGIIIKKCzus..............",
        "......................................hgg......ggszILLGGIsg............",
        "...................................................iEKLLNLIzsg.........",
        "....................................................nEGLMNNKEzg........",
        "......................................................nELNNNLGzg.......",
        ".........................................g.d........xngxEKNNNNIEg......",
        "........................................vb..g........ixiEENNNNNGzg.....",
        ".......................................pka..f.........gisEKNNNNLEn.....",
        ".................................Jw...ckbecaaa.........hgiENNNNNIEg....",
        "...............................MMlM...ckaeeeba.........vigGKNNNNLEz....",
        "..............................rwMMNM..bebceshb.........gxgzELNNNLGEg...",
        "..............................JBBJMNMMpefrhea...........isnELNNNNKEz...",
        "..............................MvvvvpJJpfeHlDa...........gEnELNNNNLEz...",
        "............................eajfgsssichceyyDa...........gEzELNNNNNGz...",
        "...............is.........bnibibheisihebcrmj............gEEELNNNNNGz...",
        "..............hxg..kol..blehcccbcfhhhfaccaaed...........gEKGNNNNNLEz...",
        ".............gog..ns...bhhefcaaabaagccffabeisd.........peELGNNNNNKEz...",
        "...........g.hIi..Ck.aHccfaaa.....aahhagvlcheca........liELLNNNNNKEg...",
        "........o.p..gsKugCqjabwad........bbesijbbaadca.......hpnINNNNNNNEz....",
        "........vgsk.xKLzKsookcagg........abesevrpaabcb.......pizKNNNNNNLGg....",
        "........iixqCCNNNqqIIqggghg.......beeeblhs.bbbcb.....niCELKLNNNNGu.....",
        "........FgizLNxqKzCCeqookk........bbhehbcb..aabii...nsiEILKNNNNKzi.....",
        "........CinKMJuLB....CCig........bbbhehf.....dfel..nsgzINKNNNNKEs......",
        "........nusNNFLJ................dcbbchfb...bbdbfegxxgizLKINNNLEzg......",
        ".......hhCKNNFs................abbfhcbbb..llggddlrggizKKKNNNLzz........",
        ".......gokKNNLh.............da.egebhhvbf.dpecggjDwahGIGLNNLKzzg........",
        "........puINNNh.............llaneeeaclgcblfcbdjgdehzEILNNLzEsg.........",
        ".........oCNNNp.............haneeebbbbaaahbaacffeiCGKNLLKEsg...........",
        ".........hqKNNMh............jlhhhbeeebcblaaafeeFIKKLLKznid.............",
        "..........hqKNNMlgggh.....dahbhfbbeeecfapaaikCIIKKKGzueffc.a...........",
        "....cca....gkoNMMMMMv...bniblbbbbbeeennssuuCGGGGCxxkeffffaccba.........",
        "..bffba.......koovkg..aaheeehccccbbeeebkbbabeeecbffbfffbccbbcffba......",
        ".bffda..............acfcleeeblfffbbceeacbbafffffaffdffcffcbfacfffbdda..",
        ".ffcd..............afffcleeeehlcfbabbeaabacfffffcccbffbffcacfaffffcca..",
        "afcb.............aafffffcheeeeglbccceecabafffffffafbfcfffabacffhfffba..",
        "afc.............bcffffffbleeeeebbeececbbabfffffffbaaccffcadbfffnffpfa..",
        "aff...........dcccfffffffbbleeebecebacjbaafffffffdbaaacfabccffffhfffc..",
        ".bfd.........afcbcffffffffcaallaecheelababfffffffcfbcbaaaaffffffeeesc..",
        ".affdb....dbcfcbaffffffffbbcfaaecbchlabbbafffffffcfcffffcffeeecppfffe..",
        ".abfffabaafffbabaffffffffcaffdbebaaaacbbccbfffffbbfaffffafevfxfbpbffi..",
        "...bcffffffcaa..afffffffccaaccbaabcaffabffbffffbacfdcffbaffefeffffffc..",
        "....abbbbaaa...dcfffffffbffbbeeaaacbfcbffffbbbcfbcffaccbafffffffffbffb.",
        "...............bffffffffbffcbeeabbcbbaffffffffffacfffabcaaffffccfffbba.",
        "...............cfffffffcbbaabeeabbbcaafffbffffffafffffaaaabfffccfffcdc.",
        "..............dcfffffffaaaaaaeeaaaafbcfffcbbffbabffffeffcaaabacabfffcd.",
        "............babfffffffbaaaaaaacaaaaffbccfffaaaabbfffizecaadca.bccaaaa..",
        "...........bffffffffffa..aaaaaaaaaacfabccfffdaaabcffeibaabccb..acfff...",
        "...........bfffffffffba..aaaaaaba.aafcbfbffffaaaccbfcaaaacccd...acab...",
        "..........affabcfffbaa..abbaaaba......ddfcfffbaaaa...daaaacffb.........",
        "..........affaaaaaaa...abbbbbbb........bcffffb.........adbbfffb........",
        ".........bfffa.........abbbbbd..........dcfffba.........abccfffa.......",
        ".........affc..........dabbba............accffc..........abbbfffb......",
        ".........cfca............abba.............affffb..........daaccbca.....",
        "........affba............abbbaa............affffb...........aachfffad..",
        "........affcd.............abbbbaa...........cfffbad..........abcfffcdad",
        "........afffcb.............abbbba...........affbhcfa..........aacffccda",
        ".........dfbcda.............................acfchcfda...........abbabaa",
        ".........aaaaaa..............................acdbbdba..................",
    ];
    const MOONA_STORM_ROWS: &[&str] = &[
        "......................................eI....................................",
        "......................................gi......oooooooooooh..................",
        "...........................x..............hqquuuuuuuuuuuuqqoh.....pon.......",
        "..........................BIs...........lnuCIIKLLLNNNMNLLLKFqol....sEs......",
        "..........................gze..........oKIIzuussqqsGLLNNNNLLNKqo...gGEo.....",
        "....................................ooqqiihh....hhinuzGLNNNNNNNJqo..szo.....",
        "...................................kqo..............gqqGKNNNNNNNKuv...g.....",
        "..................................oog.................gsGILNNNNNNKzn........",
        "..................................g...............gxxsggluCKNNMNNNKzi.......",
        ".....................................................sxi..kEKNNNNNNKzi......",
        "..............J.......................................gx...nEKNNNNNNKGn.....",
        "..............g.............................................sEKNNNNNNCuh....",
        ".............l........................g...gg.................zENNNNNNNCv....",
        "......................ho.....gx.....goFhkvll...............i.nEKNNMNNNLEk...",
        ".......gFg........hhoh.h.....xx.....ooisukgg...............oggzGNNNNNNLGs...",
        "......iGC......kkoql...............sCnzLKKKKzsp............hqjnGNNNNNNNKzh..",
        "......ivg....giuqqg.....nqs.gpl...gFIILNNNNNLIsk............qohGzNNNNNNLGo..",
        ".............qsCo......nGIvkp.....hJLNNJJJJNNNLzk...........qohGENNNNNNNGu..",
        "............quEn......iGigggggg...hJNJoqqGIIGJNLuh..........qukGENNNNNNNIsh.",
        "...........ouECh....gouGggggCIICkgiIuqzzxgsssoJNIk..........qCoGENNNNNNNLGq.",
        ".....x..glhqGzq..gouuKKKxnuzGIIIFCCFqCi.......oJLq.....J....qGCGENNNNNNNNEqo",
        "....px.hj.hqGuhlsCIKLKzLNNCIkeeeboooqz.........oNJl.....vg..qGzEGNNNNNNNLEuo",
        "......go..kzEuuCINNNNKCLNJekfrrlbchkqug........hkJv.....nx..oEEEGNNNNNNNLEEo",
        "......vk..ozEEKLNNNNNNNNNixcbjrwaabkuC..........kNv........gqEEENNNNNNNNLEGo",
        "......u..hCEGLNNNNNNNNLLbhhhfcdwbcooF...........hFp........oGzEELNNNNNNNCqGs",
        ".....sz..uGLNNNNNNNIIIEEdcevvbjbb.g...d.........ho........oCGuEINNNNNNNNGquG",
        ".....si.ozKNNNNNNLGEEEEIIIvopbffa....db.........hp.......gqGsGELNNNNNNNLGquE",
        "....gzh.uKNNNNNNIEEEGqqqqqzukbffa...beb..bb..............hqIkGINNNNNNLNKukiG",
        "....izhszLNNNNNKEEGuqqqqFIzijccff...xeaa..b..............kqnuELNNNNNNLLzkgiE",
        "..l.nzoGLNNNNIIEzqqqqCzsig...bfhh..lFeeecbhd......gp...hoohsGKNNNNNNNNKzggiz",
        "..p.nGuGNNNNNGEEqqquCng......bfh...ieeeesee.......IC...ohgizGLNNNNNNNKGuggkq",
        ".gl.nEEKNNNNLEGqquCg....x.....bgaa.ceebbeie......CGn.....izGNNNNNNNNLECo.gk.",
        ".k..kzGLNNNNGGqqqs.....xg.....bbccbgehdwbwa..........FF.vGINNKzNNNNKEEqh.p..",
        ".lh.nzGNNNNLEuqqCg.............abeclbheDDDa...........hsGKNNNLLNNLKEEzq.....",
        ".kogCELNNNNKEqqGi.............JMbecbnhhyjyb.........houGKNNNNNNNLIGuGzo...lk",
        ".kCgCENNNNLEGqCz....glk.....JBBMabbbiecwyj........gouCKNNMNNNNKEEskuIqh..gGq",
        ".hCkGKNNNNLEqzGu....sxg....jglMMBbcbgdbaaa....hhooquzLNNNNNNNKEEsgsuzh...ICl",
        ".hCkGKNNNNLEqzso...gCi......MNJvvabcbdabbcdjssuuzLLLNNNNLLLIEEEu.hGuq...ssp.",
        ".iGnGKNNNNLGqCno...ix.......MBBBvpaaaagbwchbqCIIIIIICGGGEEEsuIu..sCh........",
        "..szGKNNNNLGqkgk..gzg.......MvlvllabaeiBccbcgkqququqqqqzqqokkk..ok..........",
        "..qEEKNNNNLGqk.....zg........B....laaeievvvbd.ggggghhkJFh...hp..g.....pg....",
        "..izEKNNNNLGqk.....sg........l.....aiciieiie.......lhhhi..lk....ph...pKx....",
        "..gqEKNNNNLGqk......................pbeecchb.....ig...........hoog...gFp....",
        "...qEKNNNNLEqk......................acbbhhbad...lsv..........hl.............",
        "gv.ouGNNNNLEqo.......................bcfpecccg..............................",
        ".l.ouELNNNLEqk..iz..................acceehbffblla...aaaaaaa.................",
        "..lhqzKNNNNKqk..gui................alhbcceabcleea..acfcbccbbba..............",
        "....hqGNNNNLGk...iC................dehhhkccaagcbbbfbffbffbfcfccddd..........",
        ".....qzNNNNNGkg...ggg..............pebbhklcagcbafffcfbffaffbdadffcd.........",
        "...g.kuKNNNNIuk....gB...........ddiieeaahbabgbabffcffffcfffdacacfffadda.....",
        "..gx..qCLNNNKzog................eexeegcbgbcbhabcacafaffdfffcabcaffffbca.....",
        "...ix..ozLNNNzqo.......n.......leiehfeeeafbpbaaffaafbffcffaadbfffifefba.....",
        "....eladuINNNNzqh.....gCg......lalebbeeeecblbbffffaffacfffabacffffffffa.....",
        ".....cfckuLNNMNFo.....gng.....jjaaaabceeebcjafffffcbcfaabaacffffffFhepb.....",
        "....affcgquLNNNKqoh.........aallcbbbbbeeecjafffffffabcaaaaacffffccefxec.....",
        "...bfff..gouINNNLuqo......dleeeclffcbabebbaafffffffcaaafffdfBfseaxpcccB.....",
        "...dffa...hkqKLNNLFqohg.dableeeeccffcbabeeaaffffffffccfffdffffeffbbfffl.....",
        "...dffa.....hqqLNNNLLuokkkhleeeeelcfbcaeeeaafffffffffcffcadffpffffffffc.....",
        "...dffa.......eoosLNNNNMFohaleeeeeclbeaceblacffffffdccbbbaafffffffffcffd....",
        "...dafa.........ghkkkkkhhfffcheeeeeaeefbahacafffffbaaffbcaaaffffcffffaaa....",
        "....aff............acbbfffffbcleeeeceehbhlacacfffbacaffdaabaacffabfffbca....",
        "....acfba.......dafcbbfffffffbadjhcebccelabaaafbacffafffaabaaabbaafccbd.....",
        ".....afffa....aacffcacfffffffccaaahebblaacabfcabffffaffffccccabcaaBjaB......",
        "......affffffffffcbaafffffffcacfcaecabbaccaffffffffbaffffffffa.aalaal.......",
        ".......abcfffffcba..bfffffffcabfbabbabcbffafffcbffbabfffsxffbd.baBaag.......",
        "..........abba.....bffffffffbdfaeeeabbfbbbccffffbaaaafffeifacc.ccccc........",
        "...................affffffffacfaeeeabbfbaafccfffffaaafffffcacc..bffb........",
        "...................bfffffffcaffccecaaacfaaffccfcbcffabfffcaacc..............",
        "...................cfffffffbcffaaeeaaaacfaaffaaffffffdabbaaacc..............",
        "..................afffffffcacaaaaccaaaabcbaaacfffffffffcaaabcfd.............",
        "................aaffffffffaaaaaaaaaabaaafcaaabfffffffffabaacbfd.............",
        "...............cfffffffffbaabaaaaaaaa..abbfdaabbfffffbffffaacdacb...........",
        "..............afcffffffcaaacbbbbaaad......ddd..aabfffcffffcadacffa..........",
        "..............cfabccccaaababbbbbbaaa...............ddcfffffbadfffb..........",
        "..............ffaaaaaaaaaacbbbbbcca..................bffffffadffff..........",
        ".............ffbaabbbbb..dbbbabad.....................affffcfbcfffg.........",
        "............affbad.......dbbbd.........................cfcffcdaffbba........",
        "............affaa.........cbbd.........................bfcccccafbfff........",
        "............afba..........dbba..........................adab.daacfffa.......",
        "...........dffca..........abbbd..........................a.d...dbbbccb......",
        "...........fffcb...........bbbaaa...............................aabffcbd....",
        "...........cffcfa..........abbbbb................................abcfacca...",
        "...........afcgbb...........aaaaa.................................ababbaa...",
        "............ba.aa....................................................aada...",
    ];
    const MOONA_SLEEP_ROWS: &[&str] = &[
        "..........................b...................................",
        "......................h...cc..................................",
        "......................l....cb.................................",
        ".....................jb..aabld................................",
        "...................aahfabhhahg................................",
        "............dcccbadcclcceehhhb...ddd..d.......................",
        "..........acfffffcbbcgccfhiicbllavssssiaaaa...................",
        "........ddbfffbbfffcbafcghbwDceeeeeeeepdffffffaaa.............",
        ".......aaafffffffbffaaefbyyDddeeeeeeeelbfffffffffa............",
        "......acccccccffffbfcbbfbryyyrvpeeeeeepdffffffffffca..........",
        "......ffcafffffabfbccbbehryyyrJMeeeeelbcffffffffffffa.........",
        ".....dfffffffffffbfbalMcbhDyybhMleeeepacffffccfffffffd........",
        "....dcbffffffffffbbbaHJebcaaabebleeehlbffcabffffffffffa.......",
        "...ddcabbfbccccccbabcaehhbbbbbehlclpjacfdacffffffffffffd......",
        "...dcccffaffffffffcfcaxshhlllnbacgaabcfcbffffffffffffffb......",
        "...bcbbaffffffffffffaaxnhegpxeBlgaabbffbbfffffffffffffffb.....",
        "...bbdcaaffffffpffffabsiecliihhiibabbffabfffffffffffffffdb....",
        "...bcbcbbfffehfffffffbecfaeinhbcbbdbcbbabfffffffffffffffbfa...",
        "..affabfffffffeefikffhfjabbcecehadcbbbaabfffffffffffffffbff...",
        "..afbaafffffffepfxeffcabbaehhchhaeebbaeecbffffffffffffffbffb..",
        "..acfbafffffffffffffaaaecaabehheahhcaeeeeeaffffffffffffbabffa.",
        "..abfbafffffffffffbbfcaecaabccebrccbeecbeeaffffffcffffccabffga",
        "..abcbaephffaaaffffffcaccaaabacyyraeeebaeeadffffffbfffcbaafffa",
        "..bbaaacfffffffffffcbdecaacfhhbyyacecbbababacffffffbbcbbj.fffa",
        ".acfbbbcfpffffffffffaaelhcaabbbaabbbbbaceeecbbffffffcaaa..fffa",
        ".acfcbaabfffffffbcccdaaikcacehfbbaabbaaafheeabbffffffcad.affca",
        ".cfbaaaaaabbbfffaadacbalkhacehbeeeecaaaaceeeaaabbbffffb..affba",
        "acbfffhbbaaaabffffbfbbbbihacehaceeeeeeeeaceegbfffffffbadcffca.",
        "abbfffbaabffbbaccbaaaffbhfabefabbeeeeeeeeaceebccbbbbbbaafffba.",
        "abcfcbbffffffcbadcfbbfcbawwdefbbbbbbbbeecacceebbbaaaacffffba..",
        ".abcbbcbcfffffcbaaaccbbaaDDybabbbbbbbabaaebaeeaaaafffffffba...",
        "..abadbffbfffbbbbbbbaad.armjdd.aaaccfabheecbceafffffffcba.....",
        "....abbfacfcbbaaaaaaa...........acffffbabeeeabedacfcbaad......",
        ".....aababbbd...................bbcbbcbaaaaaaceeegaadd........",
        "......abaaa......................baaaa......abbchgca..........",
    ];
    const MOONA_BARE_ROWS: &[&str] = &[
        "......................................d.dd...............................",
        "......................................p..b...............................",
        ".....................................pb..dd..............................",
        "....................................inab.ac..............................",
        "....................................phecedf..............................",
        "....................................phehpee..............................",
        "................................j...ehcaiib..............................",
        "..............................jrBJwJgedwphb..............................",
        ".............................rjlNNNNrclywya..............................",
        "..............................MMNNJJvheyyya..............................",
        ".............................MBvBJhbbaahyw...............................",
        ".............................Mvvvcehcccbbaa..............................",
        ".............................MrgesCeabcbbbeb.............................",
        "...............................dxneeajceiheh.............................",
        "...............................bcefajilpbwahc............................",
        "...............................eefaaesnhpkpba............................",
        "...............................bcbagbeihekkb.............................",
        ".............................dbicb.aeeechbcbd............................",
        ".............................lbecd.dcccfhcabcdd..........................",
        "............................biebg...ccehhcabgfwb.........................",
        "............................gihc...accbehbdaacyw....bb..bad..............",
        "............................ghfa..dghhaecadgbaja..aaffcacfbacba..........",
        "...........................behcd..dcfffenbdggcbabccbffafffbffccaa........",
        ".........................hhchb...bjecbflxbbcfbacffcffbfffbffbcaccfcd.....",
        "..........................hbdbjdgeeefbabgbclcacfffbfcaffcfffcafacfffbdca.",
        "..........................cyywbpbneecbeccbahaaccbdffccffbfffcacfbffffbda.",
        "..........................hffbeeiiehceecfclcabcfcacfdffbfffaadbfffpffgba.",
        "...........................kkhkkhebbeeeebcpcaffffbgfcbfbfffaccfffffffffb.",
        ".............................kkbbaabbeeeebjacfffffaffcccdfaafffffffBepBb.",
        ".............................eebbcbbbceeeagaffffffbabbaaaabafffffbbefxec.",
        "..........................iiieelfffbbbeecaaffffffffabaaffffffhexkcCFaffi.",
        ".........................aleeeefhffcbabeebaffffffffafaffffbffefepfccfff..",
        ".........................aleeeeellfbcaceebaffffffffafccfffacffvffffffffb.",
        "........................ffjeeeeeecjacfeeccacfffffffaffadbcbafffffffffdfcd",
        ".bcd....................hfblheeeeebcefaahgcbfffffffafffabcaaaffffcffffacd",
        ".ffc....................khfagheeeeceefchhacccfffffbafffcaabbaadffaaffffc.",
        "dffa...........af........hfcaabdeececchhbaacbffffbbbfffffaaaa..adaaabbab.",
        "cffa..........affbbbbbbbbfhbffbfbaebahhabacbcffcabffffffffffb....caBara..",
        "ffc..........bcfbafffffffffffffhcccbababfbabfbbfffffffffefffa....dffga...",
        "ffc.........dffcaaffffffffffffhfbcbabfbcfbacfffffffbfffiFefbd.....aba....",
        "dffd......dacfcadafffffffffffcabegabbffbcaacffffffcafffexcbfd............",
        ".bffa...baffffa.dbfffffffcccccaaeebbbcfbbaaccfffffbafffffafc.............",
        ".afffcccfffcaa..dcfffffffcdcaabaeeaabaffbacfcbcffbabffffaafa.............",
        "..abfffffbbad...acfffffffaaaaaaaeeaaaacfaacffffcbaabcbaaaaba.............",
        "....aabbba......bbffffffcaaaaaaaceabaaaffabbffffaaabaaaaabfcd............",
        "...............abcffffffa..aaaaaaaaaadabfbafffffaaaaaaaaabff.............",
        ".............ddbcffffffbb..aaaaaaab...dabacfffffadd..aaccbffb............",
        "............daffffffffba..dabbaabdd.......afffffd.....aabbfffa...........",
        "............dffcffffcba...dbbbbbcb........dffcffd......abcbffd...........",
        "............dffcbbbaaa....abbbbca.........dbfcffd.......babffcd..........",
        "............cffaad........bbbba............affccd........aabbcb..........",
        "............cffba.........bbbba............acbffb.........aacffa.........",
        "............cffbd.........bbbbb............abfffb.........dafffa.........",
        "............fffbd..........abbba...........acfffd..........bfffa.........",
        "...........afffa............abbc............bfffc..........abcfa.........",
        "...........afffa............abbbda..........bcfffc........babcfd.........",
        "...........afffcda...........abbbbba........bffffbbd......abcda..........",
        "...........affffb............aabbbba.........cffbcca......bbbd...........",
        "............bbbbad...........................bcfacab......dddd...........",
        "..............................................aaaaaa.....................",
    ];
    const MOONA_EAT_ROWS: &[&str] = &[
        ".................................g..bg.....................................",
        "................................da...a.....................................",
        "................................xa...ah....................................",
        "...............................bpaaadaha...................................",
        "..............................dJhdeebaha...................................",
        "..............................dBhceeecha...................................",
        "..............................dehcehscia...................................",
        "..............................dghbdbhcb....................................",
        "..............................llcbrjyga....................................",
        "...........................B..JlecrHDHd....................................",
        "............................MMJechjDrra....................................",
        "...........................JMJhebbjrmtAra..................................",
        "..........................wBMeihhdadjmmdry.................................",
        "...........................JrssnhafaajmjbDw................................",
        "..........................dpesnhhhbbpbcbbwj................................",
        "...........................dbeehcbhhxbbaacb................................",
        "...........................jjhbabiehyppbaffa...............................",
        "...........................aaaaaciniheiaaffa...............................",
        "...........................bebaahcccceeaafha...............................",
        "..........................acbbgdeawmhebaacga...............................",
        "..........................agiicdddAAaeb.aaa................................",
        "..........................deihhcaammbf...aa................................",
        "............................baefcyrcca.....................................",
        "..............................cbbwwfbfg..aa................................",
        "............................dbcbawwbnbhadhhb...............................",
        "..........................bdrbiecadfvbcghheea.daaaaaaj.....................",
        "..........................apapeccebbbbbahbbaaffffbfffbccb..................",
        "..........................araihhceegfcbdeaacffffbfffcffffb.................",
        "..........................albehbeeeebcarbabcfffcfffffbbfcba................",
        ".......................aaaapaaabbeeeeaagabbcbffbffbffccaccffa...a..........",
        ".................dbcccapnnelhcccbbeeeepbadbfdcbffbffffacafffffbff..........",
        "................afffffleeeebphccbbbcaaaaaafffaaffaffbcaccaffffcbd..........",
        "..............bfffffffaleeeebhfcbaaceeaabffffcaccbfbaaabfffiffifa..........",
        ".............bffffffffcleeeeeajracaeeeaaffffffafcbcabaabfffefffff..........",
        "............bffffffcbcfcheeeeecaceacccjcffffffbadabaafffffffhhfhe..........",
        "............fffffffffbfblleeeeeachhbbljfffffffcaabcbafffffffhhheea.........",
        "..........dbffffffffffccbadlleeeehfllcbffffffffabffccfkeiebxccfeca.........",
        "..........bffffffffffffbfcaalpgeecehgacfffffffcbbffaffpcnefexbffse.........",
        ".........dcffffffffffffbcfcbbacebbljaabfffffffbcccfafffnffffcbfffa.........",
        ".........abffffffffffffbbffffaccbaabcabbfffffcbccbaacffeffffffcffb.........",
        ".......dcbbffffffffffffbbfffcebaabbffabacfffffffccbbaffffcffffccfca........",
        "......dffbbffffffffffffbbfffaeeeabbffabadffbffffcfbaabfffbfffffaaaa........",
        "....afffcabbffffffffffcaabbfaeeeafbfcbafadbffffbbfbaaaabadjacfffbcaj.......",
        "...bffcbbaabcffffffffcbabbbbbaeebfdbbbfffffffffbbcccbamDtmmmaccbdcwM.......",
        ".dffffba..aabbffffffbbaaaabbbaegafcbbafffcccffbaablecaAAAmmmmbmddHHM.......",
        ".bffcba....aabffffcbbaaaaabbbbadaffabafffffcbbbaabcfcatttAmmmmmddbdHj......",
        "afffa......abcfffbaaaaabbbbdbaaaacfbaacffffcbfcaaabbaaAAtAmmddddacbaa......",
        "bfff......abbffffffffaaaaaaacbaabacfaaccffbfffffcaaaaattAmddddaacffcba.....",
        "bcffb......abbfffffffffcddaaab.ddabcaafcbfffffffffccdaaAtddddbbbbccfcfcbb..",
        ".afffaa.....aaabbbbffffbfcbaab......aaccbfffffffffffffcaaaaaaaabbbbffcccb.a",
        "..bffffcdd.........abbbcaca...........aaafffffffffffffffbba.....daabfcaccdd",
        "...bcffffcaaaaaaaa....aaaad.............aaabbffffbfffffbcbca......aaabaaaab",
        "....bbbfffffffffffcb........................ddddddaabffcbcda.........d.....",
        "......aabbffffffffba................................abcdabaa...............",
        ".........dbbbbbbbbb..................................ddada.................",
    ];

    // Orbis the witcher, from prototypes/witcher.jpg (one art pixel is about 2.87 px there). Frames
    // 0-8 are all his standing pose (his orbs turn on a wheel behind him, drawn separately); then a seven-step walk and his other poses.
    pub const WITCHER_WALK: usize = 9;
    /// Three side views, each looking at a different orb (orange, blue, pink).
    pub const WITCHER_SIDE: usize = 16;
    /// From behind: he turns his back to sleep.
    pub const WITCHER_BACK: usize = 19;
    pub const WITCHER_CAST: usize = 20;
    /// Two frames of the orbs whirling round him.
    pub const WITCHER_WHIRL: usize = 21;
    pub const WITCHER_THROW: usize = 23;
    pub const WITCHER_BURST: usize = 24;
    pub const WITCHER_AFTER: usize = 25;
    pub const WITCHER_GHOST: usize = 26;
    /// Where an orb leaves his hand in the throw pose: art pixels forward of his centre, and up.
    pub const WITCHER_HAND: (f64, f64) = (14.0, 38.0);

    pub fn witcher() -> Picture {
        let grid = |rows: &[&str]| -> Vec<Vec<u8>> { rows.iter().map(|r| r.as_bytes().to_vec()).collect() };
        let (stand, anchor) = WITCHER_STAND;
        let mut frames: Vec<(Vec<Vec<u8>>, f64)> = (0..9).map(|_| (grid(stand), anchor)).collect();
        frames.extend(WITCHER_EXTRA.iter().map(|&(rows, a)| (grid(rows), a)));
        assert_eq!(frames.len(), WITCHER_GHOST + 1, "witcher frame indices out of step");
        let mut pic = Picture::from_frames(frames, 1.6);
        pic.idle_sits = true;
        pic.walk = (WITCHER_WALK..WITCHER_WALK + 7).collect();
        pic.sleep = Some(WITCHER_BACK);
        pic.orb_wheel = true;
        pic
    }

    /// His colours, plus see-through ones for the orbs' trails.
    pub fn witcher_palette() -> Vec<(char, super::Color)> {
        let mut p: Vec<(char, super::Color)> = WITCHER_COLORS.iter().map(|&(k, v)| (k, super::hex(v))).collect();
        for &(k, v, a) in WITCHER_LIGHTS {
            p.push((k, super::hex(v).with_alpha(a)));
        }
        p
    }

    /// Trails of the orange, blue and pink orbs; sunlight (3 faint to @ brightest); wind (4, 5);
    /// fire (6 yellow, 7 orange, 8 red) and rock (9); moonlight (0 core, # rim, + halo).
    pub const WITCHER_LIGHTS: &[(char, u32, f64)] = &[
        ('S', 0xf0b050, 0.4), ('T', 0x78cfe4, 0.4), ('U', 0xe2a0f4, 0.4),
        ('3', 0xffc040, 0.16), ('2', 0xffd060, 0.3), ('1', 0xffe490, 0.5), ('@', 0xfff8d8, 0.75),
        ('4', 0xe4ecf6, 0.6), ('5', 0xa8b8d0, 0.38),
        ('6', 0xffe070, 1.0), ('7', 0xff8a20, 1.0), ('8', 0xd8401c, 1.0), ('9', 0x4a2a20, 1.0),
        ('0', 0xf2f8ff, 0.85), ('#', 0xa8d0ff, 0.55), ('+', 0xcfe4ff, 0.25),
    ];
    pub const WITCHER_ORBS: [&[&str]; 3] = [WITCHER_ORB_ORANGE, WITCHER_ORB_BLUE, WITCHER_ORB_PINK];


    pub const WITCHER_COLORS: &[(char, u32)] = &[
        ('a', 0x150c1a), ('b', 0x221733), ('c', 0x35140f), ('d', 0x2e2045), ('e', 0x521f18), ('f', 0x551838),
        ('g', 0x412855), ('h', 0x681b4f), ('i', 0x3c3c66), ('j', 0x68372b), ('k', 0x7b2463), ('l', 0x4e4f81),
        ('m', 0x2e627e), ('n', 0xa83073), ('o', 0x7f4f43), ('p', 0x5d5f99), ('q', 0x8f4c8d), ('r', 0x4783a1),
        ('s', 0xa16a3c), ('t', 0x996b5e), ('u', 0x6e72b1), ('v', 0xaf59b3), ('w', 0x818bc7), ('x', 0xbd884f),
        ('y', 0xb58774), ('z', 0x60a5c9), ('A', 0xd181da), ('B', 0x93a3dd), ('C', 0xcea56b), ('D', 0xd9a553),
        ('E', 0xd0a78a), ('F', 0x78cfe4), ('G', 0xafc6c7), ('H', 0xe4bb75), ('I', 0xf0a2f4), ('J', 0xe2bc95),
        ('K', 0xf1ce8c), ('L', 0xf1cdac), ('M', 0xaff0f5), ('N', 0xfccbfc), ('O', 0xfbe69f), ('P', 0xfbe9be),
        ('Q', 0xd8fbfc), ('R', 0xfdf4d9),
    ];

    /// (pose, anchor column) in frame order after the nine standard frames (all the standing pose).
    pub const WITCHER_STAND: (&[&str], f64) = (WITCHER_STAND_ROWS, 32.8);
    pub const WITCHER_EXTRA: &[(&[&str], f64)] = &[
        (WITCHER_WALK1_ROWS, 25.4),
        (WITCHER_WALK2_ROWS, 30.7),
        (WITCHER_WALK3_ROWS, 29.0),
        (WITCHER_WALK4_ROWS, 30.2),
        (WITCHER_WALK5_ROWS, 32.5),
        (WITCHER_WALK6_ROWS, 24.4),
        (WITCHER_WALK7_ROWS, 32.2),
        (WITCHER_SIDE1_ROWS, 16.1),
        (WITCHER_SIDE2_ROWS, 15.0),
        (WITCHER_SIDE3_ROWS, 16.4),
        (WITCHER_BACK1_ROWS, 29.1),
        (WITCHER_CAST1_ROWS, 32.8),
        (WITCHER_WHIRL1_ROWS, 36.8),
        (WITCHER_WHIRL2_ROWS, 31.8),
        (WITCHER_THROW_ROWS, 46.8),
        (WITCHER_BURST_ROWS, 29.8),
        (WITCHER_AFTER_ROWS, 31.8),
        (WITCHER_GHOST1_ROWS, 30.8),
    ];

    /// Orange, blue and pink: the orbs he casts.
    pub const WITCHER_ORB_ORANGE: &[&str] = &[
        ".....t...",
        "....ts...",
        "..ctxse..",
        "..yxxstc.",
        "coDOPPxxc",
        "yxODDDOxs",
        "tPRODPROx",
        "tOPDOPJRx",
        "tKODOPKOx",
        "jsKKORKDs",
        ".csKPPCjc",
        "..exxxs.E",
    ];
    pub const WITCHER_ORB_BLUE: &[&str] = &[
        "...ga....",
        "...ll....",
        "....a....",
        "...bzb...",
        ".arzMzma.",
        ".mrQQFrm.",
        "mrFQQFFzm",
        "mMQQQQQQr",
        "mFQQMQQMr",
        "mzMMMQFFr",
        ".mFMMQzmb",
        "..mMMzrd.",
        "...bbbb..",
    ];
    pub const WITCHER_ORB_PINK: &[&str] = &[
        ".......g...",
        "....haqa...",
        "...gqfqc...",
        "...qqqqka..",
        "..qvIINvvq.",
        ".hqINNIAvq.",
        "avvANNIIAqa",
        "qvIIANNIIqa",
        "kvAANNIIAqa",
        ".kvAINAIqa.",
        "..qqIAIqq..",
        "...bkkkk...",
        "......a....",
    ];

    pub const WITCHER_STAND_ROWS: &[&str] = &[
        "......................c.....PPOP.....a........................",
        "......................Ej.c.JOOOORaacoE........................",
        "......................coycEOOOOOPJjjEc........................",
        ".....................jajCxPKHOHOOPjDHco.......................",
        "..................EyEyyjDxPJCEJCKPHDxEELOyc...................",
        "..................EJLnnLxxPEPOOKHPDDEEnLLLj...................",
        "..................tnnnnnttOKKKKKKODHLnnnkJe...................",
        "..................tkknnnntKKKKKKKODxnnnnE.....................",
        "...................LLknnntOoOKKOxOxynnnJL.....................",
        "...........tE......tECyykLCxxCCxxHPJJHKHt......E..............",
        "...........oOP..oOPJECKLoLKKHjeHOKKCKKOPJEEa..JJc.............",
        "...........oOPPaoOPPLHKKCKKOHyhHOHHHOPRPRROjcLRKc.............",
        "..........csCPRRxOPPPOKKCDHOHyjHOHHOPPPRRROPRRPOeca...........",
        ".........cJKOCHPPHOPPOOKKDHOHCjHOHHOPPPRRHCPRPKKKJE...........",
        ".........acxxxxPOHEPPJCHLjsOKLLOOOOOPRRRRCHKOCxCxj............",
        ".........aa...xCEecPPLJCLnnLPKOPOPPPRRRPJceyKcacc.............",
        ".........a.....acEfeJRRECJHKHHDDHHRRRRRLoffhLcbbg.............",
        ".........a.....acykyJRRJDDsxxsxxxoJRRRRPtkkhhEcdd.............",
        ".........c.....aeykLLRRLDHfjssssofLJRRRPEykkkEebb.......e.....",
        ".........c....ojoykhtLPPKEhfCKKHjhtCKRRyohkkhCcEja.....Ly.....",
        ".........a...eHsxChkhyCCycfEjeeeHexxCERRhhkkjHeECj...jLLc.....",
        ".........aaaeDHsxDLhhfaaafcxHHDHDeJHJJRRRhgtKDccHHKHLefffLL...",
        ".........LJjLHjsssHhhgaaaeDsDDDDDKHJJJRRRRttKDcbosoxoffoEEEJ..",
        ".........ffhtHKxssDJaaaaaCxsDDDDDHOPJJJRRRRPHxcfoxoHJEEEEEE...",
        ".......EoeffjooetsHcaaaayCOKCCDDDOLRJJJJRRRyCDcjjtj...........",
        "......tJJyfcLcbbeEEabaaeJHKOPPHHCRRRJJJJJRRioxcbbta...........",
        "......JLJEyyabbbbbbaaaJHyfKOPPPPRRRRJJJJJJRddttbbbb...........",
        "............abdbbbabaJHJffLOPPHHKRPLJJJJJJidbbddddd...........",
        "............abbbbbaaJsEEhhfPRROHHRPOKJJJJEEbbbdddbbd..........",
        "..........abdddabaacyJsjhffLRRPHLRPOKhhikyEbbbbdbaad..........",
        "...........dddddbbaCeLohkhhhRRRPRRLOLhhhnqtjbbbb..............",
        "..........bdddddbacEfLohkhhhRRRRRRPLEhhkqqJLbbbba.............",
        ".........abbbbbdbcEehLohkhhhRRPCHPLyhhhkqnLPabbbbb............",
        "........abba..addcJffLohkhhhRRPHHPKyhhkkkkkoLabbbb............",
        "................oJffJjhnhhhhPPPPPPKyhhkhkkqqLabbbdd...........",
        "................oCffLjhkhhhhPPPKKPKyfhhhkkkkyLabbdd...........",
        "..............b.oCffLjhkhhhhPPPDHLKyffhhhkkkkLcdbbd...........",
        "..............baoyfeLhhhhhhfPPPHCPKycffhhhkkhLcbbbbd..........",
        ".............bdjoffeLfkkhhffRRRRRJKycceeffhhfJcbbbbdb.........",
        "............bddjohfeLfhhhhfbLPRRRRPeceeeeeeeeLcbbbbbdb........",
        "...........abddfthfeLffffffcPPPHRLccceeeeeeffcLabbdbbdd.......",
        "...........bbbyjfhfJceeeeejaPPPHPJceceeeecffffJabdbbbb........",
        "........bbbbbbCefhfKceeeejbaPPPRLKceeeeecffffeJa..............",
        "....abbbbbbbbbyefhfKeeeeeeabPOLPKKceeeetefffffJ...............",
        ".....bbbbbbbbbyefofKeeeeejabcOKRJoceeecycejfffJ...............",
        "......abbbbbbbfhcLfLeeeeefabatJEo.ceee..ExEoffJ...............",
        ".......bbbbbbtfftffLeeeeej.a...J..ceee....cPPeJ...............",
        ".........bbbbyfetffLeeeej.........ceeej......LJ...............",
        ".............tffftfLceeee.........ceeeee......c...............",
        ".............eoffcELeeeee.........ceeeee......................",
        "..............jjoyxseeeee..........ccccc......................",
        "...............acoteeeeee.....................................",
        "....................eeeee.....................................",
        "....................eeeeec....................................",
        "....................ceeeee....................................",
        ".....................ccccc....................................",
    ];
    pub const WITCHER_WALK1_ROWS: &[&str] = &[
        "........................RPPPPP..........",
        "..................y....ROOOOOOL.........",
        "..................Ey..RPOOKKKOPo........",
        "..................LCJ.POOOHKKKOO........",
        "..............yCJLnnCHOKOHJEJHKOHCEE....",
        "..............ECnnnnCxOCOKJPJPKLnnnK....",
        "...............etnnnkCKCKJJKJPKLnnyj....",
        "................EtnkkjOCOJJJKJKLnts.....",
        "............E....EtJkkoHKJJJKHKLoC......",
        "............Lt...ajHxtoHHHxsCxCOLHoj.ta.",
        "............JREyRRRROKKKOHCHeHHKKKPLJJc.",
        "............HPPPCRRREEHHOOOLjCHHJRRPCE..",
        "..........yJDDPPKLRRRPLHOOPPjEHKPPRRxc..",
        "...........JHHCPCHHPPPJJKEOPjJKOPPRPya..",
        ".............ogdffjLJPOKooKPLOPPPRPL....",
        "............bdbdhhfePOPLkkCPPPPPRRLt....",
        "...........bgddhkkhhfJRRPHDDCDCHPRc.....",
        "...........gddhkkkhhfJRRLDejjjjyPPca....",
        "..........dgbhhkkkhfcELRLsjJHKEhOLfa....",
        "..........gdbkkkkkhbbcJLEttjjjhkLekj....",
        ".........ggbgkhLCohbbbcefjhLKHLhfhoPc...",
        "........gggbfoLtKHLbdbttttyCHDHxfjEje...",
        "........ggbbELejjytbbctsDxDDDDxKjojefe..",
        ".......gggbbEsehfjjdbCxsDDDDDDxHjsefff..",
        "......bgggbbysffhfbboHKKKKPPDDHHCoeffLJ.",
        "......bgggbyyefhhfaaoHKKKKPPPPRPHH.cEEEc",
        ".....agggbbtcbfhJJbdfejKKKPPLOPPHH..EEEc",
        ".....bggdbbbbbLtJLffhhhHHKPPHHRPHK..EEEe",
        ".....gggdbbbbbJHJefhhkhKefPREJRPjLc.....",
        "....dgggbbbbbbcEEefhhkJLffRRRRRPotP.....",
        "....dggdbbbbbbbaafhhkkLehhRREERPkkL.....",
        "....dggdbbbbbbbbbfhhkjLfkhPRHHRPhhoL....",
        "...dgggbbbbbbbbbfhhkkLjhkfPRPRRPhhhJ....",
        "...dggbbbbbbbbbfhhhkyLjkhfPRRRRPhhhL....",
        "..dgggbbbbbbbbfhhhhhJehhhfPPCCPLhhkJJc..",
        "..ggggbbbbbbbdhkkkkhLfhhbbLPDDPPhkkfJe..",
        ".dggggbbbbbbfhhkkkELehhhbbLPCORehkkfJj..",
        ".gggggbbbbbbfkkkkhEjffffbbJPKPEehhkfJe..",
        "gggggbbbbbbbfqhhkJsjfefbbbPPHPJefhheyE..",
        "dggggdbbbboefhhLfKoeeefbbbLPHPHceeeejE..",
        "dggggbbbbbojyohjyOceecdbbbLLJPKceeeffJ..",
        "dggggaaabbtjthhkLececbbbbbLLPPxceeeffJ..",
        ".dggg.......LfhLxcc.abbb...ERLoeeeeffJ..",
        ".bggg.......cELJcc..........R.eeeeecfL..",
        "...gg.......esoec.............eeeee.ea..",
        "...ad......aejec..............eeeee.....",
        "...........aeeee..............eeeee.....",
        "............aceec..............eeee.....",
        "...............................ceeeee...",
        "...............................cccecec..",
    ];
    pub const WITCHER_WALK2_ROWS: &[&str] = &[
        ".............................PPPPPPc.........",
        ".......................yc...POOOOOOR.........",
        ".......................xC..POOOOHKOOP........",
        ".......................yxE.POOOOKKOOP........",
        "....................EEJttsCKOOOCyLJCOCCEEj...",
        "....................ConnnJxHCOOPPJPHOknoJj...",
        "....................eytnnkoJCOKKJJHHLknJ.....",
        ".....................ctnknnCCOKHHHJHLkEt.....",
        ".................L.....yLnkkKDHHHHHHLoH......",
        ".................EJo....CstfKDHsxxxHOsHEj....",
        ".................EPJCJRRJHKOHHHCHesKKHHKPyCc.",
        ".................JRRPEPRRLCCKOOPPetKKHPRPJCc.",
        "...............cJLKKRJKPRRLLKCOPPjxHHKPRPJc..",
        "................yJHCLCHKPRPPKCCOOjxHKPRRPLa..",
        ".................aggffffELPPJssLOKOKPRRRPe...",
        ".................gggffhkoEPPLtoKOOOOPPRRa....",
        "................ggggkkkkhfCRRPJCCCCCCORc.....",
        "...............bdggkkkkkhfyRRLHjjjjjjKPc.....",
        "..............dggbhkkkkkhfyRROCohJJLexCc.....",
        "..............gggbhhkkkkfbjCJHHtqjjjeCsf.....",
        ".............dggbbhEEyhhbbbjjstfjCHHyjffEj...",
        "............gggdboEojCCybbbttCCJjCDDHjkCCoa..",
        "...........ggggbbyyjhjeybbbyssssDDDDHEoxefc..",
        "..........ggggbbcHjhhfocbbcsxxxKDDDDKKeyeffa.",
        ".........ggggdbbtofhhhabacCKKKOOPxDDOHojffyL.",
        "........ggggddbbtffhhhbbatCOKKHORPORRKejfftJL",
        ".......dgggbbbbbdbLffPjbdjeexKKPRLKRPKHcaJJHJ",
        "......dggggbbbbbbbLjLOodfhhfLjKPRHHRPCH..EJJE",
        ".....ggggggbbbbbbbJCJeffhhhJLhtRRRRRPjL......",
        "...dggggggbbbbbbbbeLLffhkhjLhkhRRRRRRfhL.....",
        "..adgggggdbbbbbbbbbaahhhhyLehkhRREJRPffL.....",
        "..bggggggbbbbbbbbbbbfhhkktLfkkhRRCCPPfhoEe...",
        ".dgggggddbbbbbbbbbdfhkkkkJokkkhRRJLKLfhfEe...",
        ".ggggggdbbbbbbbbbffhhkkhLjfkkfbPRPPKLffhLj...",
        "dggggggbbbbbbbbgfhhhkkhLHehkkdbPOJKPKfhkhE...",
        "dgggggdbbbbbbbfhhhkkkkELefhhfbbPOCKPLjhkkJ...",
        "dgggggabbbbbfhhhkkkkkyLLfhhhbbbPPHCOOefkhJ...",
        "dggggda.bbbyfhhqkkkkkLLefffgbbbPPJPPceefjJ...",
        ".gggg......yjhkkkkkyJyeefffdbbbPPPRPeeeeeJc..",
        ".aggd......yCffLffoExeeeeefbbbbPPJROeeeeeJc..",
        "...d........yjfeftCxeeeeecbbbbbPOJOOeeeeeKc..",
        "............ttefyLEeeeeecabbbbbLKPKLeeeeeJc..",
        ".............yCJLcceeec........ePPL.eeeeec...",
        "..............Lcceeeec...........Pc.eeeeea...",
        "...............cceeec............c..ceeeec...",
        "................ceeec................ceeec...",
        "................ceeec................cceeeeee",
        ".................eeee................cceeeeec",
        "..................ccc................cceeeec.",
        "......................................ccc....",
    ];
    pub const WITCHER_WALK3_ROWS: &[&str] = &[
        "...........................aPPPPP..........",
        "......................t....POOOOOP.........",
        ".....................oCL..COOOKOOOJ........",
        ".....................yCHKHOOOKKKKOO........",
        "..................EJJonHDKKOOECJJKOCyyHa...",
        "..................EoknnnxKHHOELLLJOnnnEc...",
        "...................EtnnnoKHHOJJJJHPknyj....",
        "....................EknnyJHCOJJJEHPkott....",
        "...............Ec...cyyhknJDOCJJJHLoyx.....",
        "...............Eoa...jHJjhLHKxsEyxKjCCy.a..",
        "...............CPJJLRRLJKKHOHHDsxOHHKPPEE..",
        "...............CPPPHPRRJJHHJPPPtxHDHPPPHH..",
        ".............yJDCLPKHLRPLKHHJPPtxKHOPRRKc..",
        ".............osCCCCHHKPRRJHHHPPyxHKOPPPK...",
        "...............cettffyCLPPKotLPOKOORRRPc...",
        "................gfffkhoJPLJonLOOOOORRRR....",
        "...............iggkhkkheJRRPJCCCCDDKPP.....",
        "..............aghkkkkkhfERRLJojjjjeHPL.....",
        "..............ghhhffhggbEPLJCjhEJJfEJy.....",
        ".............gghfoCHfgbbcEJCEfhootfEEsCa...",
        "............ggdjCHDDCabbbfjjjfhCHCfhoCCea..",
        "...........bgdbCDjjexabbbbttyCoHHDCotjfhha.",
        "...........dgddCCjjjfbbbajsssDDDDDDxjffhfa.",
        "..........aggbyxsjjhfbbbaysssDDDHDDsjhhfJE.",
        ".........adgdbysofhffbbaLHHHOOCDDDDHoffJEEJ",
        ".........bggbbyofffLLaayCHHHOPPCDKOKeabJCCE",
        ".........ggbbbbbbLJLLaboejoKOLPPPRLOJ..eECE",
        "........gggbbbbbbJJHLagfhhECKOPJKPLeLc...t.",
        "........ggbbbbbbbEJEaghhkoJjoPPHHPLjLy.....",
        ".......gggbbbbbbbbbbdfhkhLjhfRRJHPLjjJ.....",
        "......gggdbbbbbbbbbbfhkkLjfkfPRRRRLfhLL....",
        ".....bggdbbbbbbbbbbbfkkkLehkfPRRRRLfhhL....",
        "....ggggdbbbbbbbbbgfkkkLehhhfPRKKPLfkhhJ...",
        "....ggggbbbbbbbbbfhhhkhLfhhfbPPEHPLfkhkL...",
        "..dggggdbbbbbbdhkkkkkhLefhhbbPRRRRLekkkyJ..",
        ".dgggggdbbbbdhykkkkkkjEfhhfbbPPPPRLehhheK..",
        "dgggggbbbbbbhkEokkkkkLofhhdbbPPHHRLefhfeK..",
        "ggggggbbbbbthhfqkkkhJcfhffbbbLPHHOLeffeeJ..",
        "gggggbbbbbbtfkhfhhhLjefffbbbbLRRRLceeeeeJ..",
        "dggggbbbbbboehLLffLceeeffdbbbLPHKLaeeeeeJ..",
        ".ggg.......oEefeeLeeeeefdbdbbLPHDOaeeeeeJ..",
        ".agg........yejeLceeeeefdbdbbLPCCOceeeeeJ..",
        "............tCEEcceeeea.bb...LOPLeceeee....",
        "............oEcaceeec........tOOLcceeee....",
        "...............ceeee..........eKJaceeee....",
        "..............ceeeec...........Et..ceee....",
        "..............ceeee................ccee....",
        "..............ceeee................cceeec..",
        "................ceee...............eeeeeee.",
        "................ceee...............ccceeeec",
    ];
    pub const WITCHER_WALK4_ROWS: &[&str] = &[
        ".............................RPPPPc............",
        ".......................ye...JOOOOOP............",
        ".......................EJ..EOOOOOOOP...........",
        ".......................yCL.POOOKKKOO...........",
        "...................tEEJkECLOOOEJEKHOoLtye......",
        "...................tyhnvnyKKHOELLEOOonnCc......",
        "....................yynnnnLKCOHKKJKLknEc.......",
        ".....................ykknnLHCOHHHHHLknxo.......",
        "................JE....LCykhCHKDHHHHKjxca.......",
        "................JJc....CHyfsDHxCxHCOCCCL..y....",
        "................EOPyLRRRLKHKHKOKLoyOCOOPPjC....",
        "................HPPPLJPRJHCCOOPPLoyKHKJROoy....",
        "..............JECDLPPHKRPLHHHJLPLoCKOPRRKCj....",
        "...............JCxCJJHHOPROOHHHOKoCKPRRRKo.....",
        "................cCtffeJHJRPKJsoKOKOORRRRC......",
        "................dddddkfoJPPJEojKOOOPRRRRt......",
        "...............dgggdkkkkyERRJKHCDDDCOPPje......",
        "..............dggddhkkkhhhRRRHsejjjeLPPjb......",
        ".............ddgdbfhhoLLPtRRPHjjEHHjJPyjyc.....",
        "...........addggbbghoHHJJJEJJCjtjoxjEEjtCca....",
        "..........dggggbbbfyKjJEEJejjjeoJKKjejyCjjf....",
        "........dgggggbbbbfJefJEJtbjtxJeKHHDcjysefff...",
        ".......bgggggbbbbbJcffejcacjxsxKKDDDoeyeffffE..",
        "......dgggggbbbbbbCcffoybbjssxxHHDDxHtxeffLJJLc",
        "......gggggdbbbbbbCejoEabeKKHKKKDDDxKHyefeECKJe",
        ".....dggggdbbbbbbbEcoocbtJHHHOHOODDOLKxt...EJJc",
        "....ddgggdbbbbbbbbcLEjbbcejjKOORPKPPECCo...acc.",
        "...bdggggbbbbbbbbbbdbbbbffftKOORLKPPyhLx.......",
        "..dgggggbbbbbbbbbbbbbbbfffjJjjPPJKOLhkjJx......",
        "..ggggggbbbbbbbbbbbbbbfhhfJjffLPJKJLhkkEH......",
        ".bgggggbbbbbbbbbbbbbbdfhhLyfffPRPRRJfkkkLL.....",
        "aggggggbbbbbbbbbbbbbdhhhhLjhhhPRPPRJfkkkjKc....",
        "bgggggbbbbbbbbbbbbbbhhhhLfkkkfPPJHLfhkkkhLx....",
        "bgggggbbbbbbbbbbbbdhhhhJofkkkfPRJHLcfkkkhoK....",
        "aggggdaabbbbbbbbbbhhkkkLjhhhhLPRRRLefhkkkfCJ...",
        "..gggb..bbbbbbbbbdhhkkJjfhkhhLLOKRjffhkkkfJj...",
        "..dgg....bbbbbbbfhfhkkLehhkkhPHHHPddffkkkfjE...",
        "...gg.....bbbbbbhhhhktsehkkkELHPPJcbdfhhfffE...",
        "....a.....aabyffhkhhhLceefffPPOPRoabbeffeeecJ..",
        "............btfftELhJeeeeeeePKHJJtbbcceeeeefJ..",
        ".............yfhtjhjEjeeeeeHCOKLLobbceeeeeffJ..",
        ".............eEfhhhtKjeeeeeCHPLLj...ceeeeeefJ..",
        "..............EehhjHjeeec..jCCCj....aeeeeeccE..",
        "...............JyfLxeeee....tc.......eeeee.....",
        "...............eCyKyeec...............ceeee....",
        "................eKKcec................ceeee....",
        "................ceeee..................ceeeeecc",
        "................eceee..................ceeeeecc",
        ".................cceee.................ccecc...",
        "..................ccee.................ccc.....",
    ];
    pub const WITCHER_WALK5_ROWS: &[&str] = &[
        "...............................PPPPPj......",
        ".........................y....POOOOOPa.....",
        ".........................JK..JOOOOOOOPa....",
        ".........................EHEoOOOOKKKOPa....",
        ".........................LkysOOOCCCLJPc....",
        ".......................EJyntHKCOEPPJJKxtE..",
        "........................cxosHKCOEPJJLKojC..",
        ".........................oxxDKHOCHHEJKofC..",
        ".....................Ly...oxxDHHHHHHHJoJx..",
        ".....................JPt.accExHHHxECELoHxEa",
        ".....................JPPELRRRPKKKKxjHHKJHOa",
        ".....................KOOOJRRRJJHKOJeCHPLKL.",
        "...................JJHKHHKPRPPPHCKKjHHOLCc.",
        "...................tKDHHHHLPPPLKHCKeHHPPE..",
        ".....................yyccffJKPPKosOOLORRJ..",
        ".....................dbbbffffjPOokLPPPRRE..",
        "....................dgbbghkkkkPPOOCCCCPR...",
        "..................gddbbbghkkkkjLOOejjjOP...",
        "................ggggbbbbghkhjEKsxsjjLEEj...",
        ".............dgggggdbbbbghkhJCojCJhjtjoc...",
        "...........dggggggbbbbbbghkyCjfftffhjLo....",
        "........ggggggggbbbbbbbbbghyyfhhhfyJHKc....",
        ".....gggggggggdbbbbbbbbbbbhyyfjhfhfeKEEyt..",
        "....ggggggggbbbbbbbbbbbbbbfytjCJeffeKyKHJ..",
        "...dgggggggbbbbbbbbbbbbbbbeCseCKoeCHKyHJE..",
        ".adgggggggbbbbbbbbbbbbbbbaJtysKHKJKxHKJEc..",
        "abdgggggdbbbbbbbbbbbbbbbbbefsCjJLPPPHKa....",
        "bgggggdbbbbbbbbbbbbbbbbbdfhhtsfJLRPPKLa....",
        "dggggda.abbbbbbbbbbbbbbdfhhtCjhJPLKKxLa....",
        "ggggd.....bbbbbbbbbbbbbffhhttkhJPJHHxKc....",
        "gggga.........bbbbbbbbgfhhjLthhLPPLLtJO....",
        "gggg..........abbbbbbgfhhfLehkhLPRRRtfL....",
        "gggd............abbbffhfftchkkhLPHKRohLc...",
        "ggg.............abgfhhfotcfhkkhLPHDChhPc...",
        "agg.............fhhkkjLttffhkkhLPPPLhkkE...",
        "..d...........affhhkjLeffhhhhhhLPPPLhhkE...",
        "..a..........jkhEyfELofffhhhhhfLHCOJhkkE...",
        "............yJhhPojLchfeeffhhhfLHDOefkhE...",
        "............yJjhheLefheeeeffffJLPLOceeeE...",
        ".............oEffxykffeceeeeffLKOLceeeeC...",
        "..............JCKyhheeeeeeeeeeLJHOceeeeE...",
        "...............JyfffeeeeeeccecPPPLeeeeeE...",
        ".................fceeeeeec....PPPjeeeeeE...",
        ".................cceeeeec.....OOPeeeeeeE...",
        "................aceeee........jJEeeeeee....",
        "................aceeec........eCoceeec.....",
        ".................ceec............ceeec.....",
        ".................aeec............ceeeea....",
        "..................ccca...........cceeeee...",
        "...................c.............ceeeeee...",
    ];
    pub const WITCHER_WALK6_ROWS: &[&str] = &[
        ".......................RPPPPj..........",
        ".................E....LOOOOOPc.........",
        ".................JEc.OOOOKOOOPa........",
        ".................LEE.OOOHHKKOPa........",
        "................etkyCOOOJEEJEPc........",
        "...............yLEnkHOCOJPPLEKyEE......",
        "................txoxHOCOJLLJJLtnE......",
        ".................ooCDOKKHJHJHJnkE......",
        ".............E...ccxDDKHCDHHHJoJC......",
        ".............JR...cosHsKHooCHJoHxtj....",
        ".............JPPxLRRPPPxKKsjCHJJHHo....",
        "............cHPPHERRRPOPOKKoCHPRLLo....",
        "...........PJHHOPPOLPOOKKLKtJHPRE......",
        "...........LCDHCCCKKPKOHJKKtKHPRE......",
        "............cEjbdbdcEKEojJKKLPRRL......",
        "..............ddddddcffhhtLOPPPRE......",
        ".............adddddddhhhkhtCHHLL.......",
        ".............ddddddddhhkkkkojeLLc......",
        "............ddddgdgdghhkkkkKCoefftLELc.",
        "............ddgggggdghhkkkkKDjffhooJLc.",
        "............ddgggggdbhhhkkkKjejhhfyJLc.",
        "...........ddggggggdbhhhhkkHjfffftLJe..",
        "...........dgggggggdbtffhkkHjjjc.......",
        "..........bdgggggggdbooojfhHsjj........",
        "..........ddgdggggbbJsjjxefHDHe........",
        ".........bdggdggggdbExsHHPRjDCea.......",
        ".........dggggggggddfjEOKPPLCxO........",
        "........bgggggggggbdfoEOKRPPPtO........",
        ".......bggggggggggbffyjPKKHJRtjL.......",
        "......gggggggggdggbfjyhPPOHHRthJ.......",
        ".....dggggggggdggdbLJyfPOPPPPqhoJ......",
        "....ddggdggggggggdfJKefPPPPPykkhy......",
        "...bgggdgggggdggdffLjffROHHLykkhf......",
        "...dgddgggggggggdfhLoffPOHHJthkkka.....",
        "..dggdggggggdggdfjLohfLOPOPyfhkkkb.....",
        "adggdggggggdgggdfLLhhfEOKLRyfhhkhf.....",
        "dggdggggggddgggfLEfhhdJOHKPtffhkkf.....",
        "dgdggggggdggggfoyeffbjLPPLJtbffhfc.....",
        "dgdgggggddgggdjJefffboORRPobbfeeeec....",
        "bdggggddgggdbfKxeecebPKPJybddeceeee....",
        ".ggggddggggdfoKjeecePPPPLcbdfeeeeee....",
        "..abdggggggffCKoecabJJPPLcbbbeeeeee....",
        "...aggggbyyojLeec....OLL.ablgceeeee....",
        ".........cyJCKcc.....Osj.....aceeee....",
        "........cejsKCc......Ea........ceeec...",
        ".......aeeeecc.................ceeeec..",
        ".......ceeeee..................ceeeeeee",
        ".......ceeeee..................ceeeeeec",
        ".........eeee...................cccec..",
        ".........ceec...................ccc....",
    ];
    pub const WITCHER_WALK7_ROWS: &[&str] = &[
        "...............................RPPPRj...........",
        ".........................yEa..JOOOOOPc..........",
        ".........................tCKcJOOOKOOOP..........",
        "........................ojnCyOOOHHHKOO..........",
        "........................tjnnKOOOHCCJEOc.........",
        "........................yxLkHODOCPPJELCEE.......",
        ".........................xEoHOCOHLLKJOtnK.......",
        ".........................ooHDKHKHJHJJLnhK.......",
        "....................Eo....oCxHKHCHHJHJoJHa......",
        "....................JPc...acEsDKKsxHHJoCHoj.....",
        "....................JPPLJPRPLLCCKOsjCDKHHHt.....",
        "....................JPRRJRRRRLPPOOJyEHOPLLt.....",
        "..................yCCCHJHORRRLLJHOJyEHOPJ.......",
        "..................tCDDCCCHPRRPPHHHKCJHPRJ.......",
        "...................cCKccccJJJPOHssKKKLRRJ.......",
        ".....................cgggdbcyJHKjoJPPRRRJ.......",
        "....................bggggdddhoyPKCHCCHKLc.......",
        "....................ggggddfhkkfEHxsejeKKt.......",
        "..................gggggdbghkkkkkJLehhjtxssjhLJJJ",
        "..................gggggbbghhkkkkJefhhjtjtoffyECE",
        "....aagggggggggggggggdbbbghhhkkkJefffLjffffffECJ",
        "..agggggggggggggggggdbbbbbdghhktCeyCEjftefcffEE.",
        ".bggggggggbbbbbbbbbbbbbbbbbdffktCssDHLfteet.....",
        "dgggggggbbbbbbbbbbbbbbbbbbbbyefjCsxxxHetjeo.....",
        "dggggdddbbbbbbbbbbbbbbbbbbbyxyxojsxxDCc.joe.....",
        "gggga..bbbbbbbbbbbbbbbbbbbbyxxxxtExxKKc.........",
        "gggg....bbbbbbbbbbbbbbbbbddeesxxJPLCPHC.........",
        "ggg.....abbbbbbbbbbbbbbbbdfhfyEoPPRPRKC.........",
        "ggd.......bbbbbbbbbbbbbbgfhhEjffPLLOPjH.........",
        "aga.......abbbbbbbbbbbbgfhhhJjhfPOCCOjJa........",
        "............bbbbbbbbbbdfhkhJyhhfLLPRRjLL........",
        "..............bbbbbbbfhhkkkLyhhfLJPRRhhJ........",
        "................abbbfhhkkhLthkhfPHHLEhhE........",
        ".................bbfhhkkkjPohkkhPHHKthhE........",
        "................dfhhhhkkjLfhhkkkPPJOthhEJ.......",
        "...............afhkkkhhjLefhhkkELRLPthktJ.......",
        ".............effhkkkkhJHjffhhkkEHJPyfhkfL.......",
        ".............oofkkkkhhLeccffhhtJHPPofhhfJ.......",
        "..............ytkhkkhLjeceeeefEJPPJjffeeJ.......",
        "..............atthhhyKeeeeeeeeEHLyceeeeeJ.......",
        "................ttjKOceeeeeeeLKKKJceeejeL.......",
        "................cLKLceeeeecctKPPLcceeejeK.......",
        "..................cceeeeee..LOPPc.cceeeeJ.......",
        "..................ceeeee....JOPJ..cceeeeL.......",
        "..................ceeec......yoc..ceeecc........",
        "..................ceeec...........ceeea.........",
        "....................eee...........ceee..........",
        "....................cee...........ceee..........",
        "..................................ceeeeec.......",
        "..................................ceeeeec.......",
    ];
    pub const WITCHER_SIDE1_ROWS: &[&str] = &[
        ".........E......ROPPPo...............",
        ".........Jjj...oOOOOOPc..............",
        "........Ejoto.yOOOOOOOJ..............",
        "........Ejkyt.OOOOOOKCE..............",
        "........CHknnCKOOCCOJCE..............",
        "........EHxykCKKOCCOKPC..............",
        ".........jxxykKHKHHOKHJR........t....",
        ".........csxCEsxHHHOKKJc.......jxt...",
        "..........cxxCCosHHOKJJ.......txDxsc.",
        "...........tCCHJHDCOKjc......esDKDxy.",
        "..........JJPPPPRRCCHJc.....txDHPOHDo",
        ".........tPLRRJCRRPJHKc.....xxKKPPOKs",
        ".........PPPROCxKPPPHCJ.....xxKKRRPKs",
        "........LLPPODxDDHOPJsJt....ysHKPRKHx",
        "........EPRPCDxxCDCHLnJP.....cDDOOHC.",
        "........ECCHDxjfhfCHHLJR......sDDDCj.",
        "........ccHHCfhkkkhjHOPR.......cccc..",
        "........ddJyfhkkkkkhKKJR.............",
        "........dgachhkknkLtHHKLGrrb.........",
        "........dgbbhkkknLKoCDDHGQFm.........",
        ".......bggbbfhknnLjjxDHJPGGzm........",
        ".......bggbbghkkLHeoxHeffjRQma.......",
        ".......dggbbbhkkKHeksxehfLJGm........",
        ".......bggbbbgkkJDeoosxyPJGGa........",
        ".......dggbbbbbfCDefCKHCGGzd.........",
        "......bdggbbbbacCxefHHHJGzd..........",
        "......ddggbbbboojxxsOPRRaa...........",
        "......ddggbbbbejosCsOPRR.............",
        "......ddggbbbdffffEjKKRR.............",
        "......dgggbbbfffhfJjJLRR.............",
        ".....ddgggbbgfhfhLcffLRR.............",
        ".....ddgggbbhhhhkJehhEER.............",
        ".....ddgggddhhkkhLekkJER.............",
        "....dddgggddhhkkkLfkkEJR.............",
        "....ddgggdbhhkkkLjhkkJJR.............",
        "....ddgggddhhkkkLehkkJJR.............",
        "...dddgggdbhhkkkLehkkfJR.............",
        "...bddgggddhhkkkLehkkhLR.............",
        "..dddggggbdhhkkkLehhhfLP.............",
        "..dddggggdbhhkkkLcffhfJP.............",
        ".bddggggdbbfhkhLjejefcJP.............",
        ".bddggggddghhhhJeeeec.EP.............",
        ".ddgggggddhhhhhJeeeea.EO.............",
        "bddgggggddhhhhhJeeeea.EP.............",
        "dddgggggbdhhhhhJeeee..cE.............",
        "bbdgggggbbffhhhJeeec...y.............",
        "..dggga.ytfffhhJeeec.................",
        "...bdda..oEEohfJeeec.................",
        "....ab...ccctyHHeeec.................",
        ".............cjjeeec.................",
        "..............eeeeeec................",
        "..............eeeeeeee...............",
        "..............ceccceee...............",
    ];
    pub const WITCHER_SIDE2_ROWS: &[&str] = &[
        "........E......PPPPPc..................",
        "........Ejc...EOOOOOPc.................",
        ".......Eooyc.EOOOOOKKJ.................",
        ".......EonEtcOOOOOOKKJ.................",
        ".......yCnnnJHKOCCOCCJ...........ga....",
        ".......yCJnnsKHOCHOJPJ...........ll....",
        "........oDxnnHHKODOJJJL...........a....",
        "........txDxtoKHKDOHJL...........bzb...",
        "..........xDCsoCHCOHJL.........arzMzma.",
        "..........CxCCJCCHKKE..........mrQQFrm.",
        ".........LJLPPRRRCDHKe........mrFQQFFzm",
        "........dPLRRPHPRPHHHe........mMQQQQQQr",
        "........PPPPLKxOOPROCJ........mFQQMQQMr",
        ".......EPPPKHxxxKHRJtKy.......mzMMMQFFr",
        ".......EPPJKDDxCHDJLnJE........mFMMQzmb",
        ".......yCxCHEffhfJCHLPL.........mMMzrd.",
        ".......ccCCefhkkkfjJKPL..........bbbb..",
        ".......bbtsefhkkkheHHOJ................",
        ".......bdffbhhkkkJHCHDC..oEa...........",
        ".......bddddfhkkkHHjCDH.jEo............",
        ".......bddddfhkkoJeExCyfEf.............",
        ".......bdggdghhhJHejCHhhhfJJ...........",
        "......dgdggdbhhhHDehCChhhLJJ...........",
        "......dgdggdbbdhHDehxCHJJJEc...........",
        "......dgdggdbbaoDDcjxHHc...............",
        "......ggdggbbbcyCDcjHHH................",
        ".....ggddggbbooesxsOORR................",
        ".....ggddggdbffoosxOORR................",
        ".....ggdgggdffhhfCeHORR................",
        ".....gddgggdffhhfCcLPRR................",
        ".....dddgggdfhhhLcffPRR................",
        ".....ddggggghhhkJehhtRR................",
        "....dddggggghkkkJfkkyRR................",
        "....dddggggghkkkLfkkyRR................",
        "....dddggggghkkLjhkkERR................",
        "...adddggggghkkKekkkEPR................",
        "...dddgggggghkhKehhkhPR................",
        "...dddgggggfhkhJehkkkPR................",
        "..ddddgggggfhkhLehhhhPR................",
        "..gddgggggffhkfLeefffPR................",
        ".bdddgggggffhkJjeeefcLP................",
        ".ddddgggggfhhkJeeeec.KP................",
        ".dddgggggghhhkJeeeec.JO................",
        "bdddggggggfhkkJeeeec.JO................",
        "bdddgggggffhhhJceec...J................",
        ".bdggggggffhhhJeee....o................",
        "..dgggggdJofhhJeee.....................",
        "...dggggdEHJjjJeee.....................",
        "....bdddbcccEJJeee.....................",
        ".............cceee.....................",
        ".............eeeeeee...................",
        ".............cceeeeee..................",
        ".............cccceeee..................",
    ];
    pub const WITCHER_SIDE3_ROWS: &[&str] = &[
        ".........E......PPPPPc................",
        ".........Ee....OOOOOOPo...............",
        "........Jjjyc.LOOOOOOOLc..............",
        "........EjkEojKOOOOOOKJc..............",
        "........EKknnCKOOHKOKCJc..............",
        "........yCCynyKOOHCOKJJc.......aya....",
        ".........cxxynHOKHCOKJJJ.......aAb....",
        ".........cxxCstKHHCOKJEj........vb....",
        "...........CDKyssHHKKJJc......aAAAkb..",
        "...........yCHHLHDDHKKc......avAAIAAq.",
        "..........JPPRPPRRHDHKe......qAINNNAvb",
        ".........tRRRRJCRRPHHHe.....avINNNIIAd",
        ".........PRRRPCxKOPPKHKc....avINNNNIAd",
        "........EPPPJDDDDCKPKxKo.....vAINNNIqb",
        "........EPPJHDDDDDHKLnLL.....avANNAAq.",
        "........EJHDDEeffjCHHLPR.......kvvvd..",
        ".........cHCCfhkkkheLOPR.......aaab...",
        "........bdyyfhkkkkkexOPJaa............",
        "........bgacfhknknJesKOJwGGi..........",
        "........dgbbfhknnnKeoHCKGMMrb.........",
        ".......bdgbbghkknLHejsHJERGww.........",
        ".......ddgbbbhkkkKDcHCofftGQw.........",
        ".......dggbbbfkkEHjexCofotPGz.........",
        ".......dggbbbbhkCHejosDHJGGB..........",
        ".......dggbbbbbfCHjfxHHJGzd...........",
        "......bdggbbbbacyCjjCDKLbba...........",
        "......ddggbbbbotetxHKPRR..............",
        "......ddggbbbbftetxoPPRR..............",
        "......ddggbbbdffffCcLPRR..............",
        "......ddggbbbffhfjCeJPRR..............",
        ".....gggggbbgfhhhLeffPRR..............",
        ".....ddgggbbfhhkhJehhyPR..............",
        ".....ddgggbdhhkkkJfhhyPR..............",
        "....dddgggbdhhkkkLfkhyPR..............",
        "....ddgggddhhkkkLehhhEOP..............",
        "....ddgggbbhhkkkLehkhEOR..............",
        "...bddgggbbghkkkLehkhcPR..............",
        "...dddgggbbhhkkkLehkkfPP..............",
        "..bddggggbbhhkkkLchhhfPP..............",
        "..dddggggbbfhhkhLchhhfLP..............",
        ".bddggggdbdhhhhEoefffcLP..............",
        ".bddggggbbghhhhHeejecaLP..............",
        ".dddggggbbhhhhhKeeeec.JO..............",
        "gddgggggdbhhhkhKeeeec.LL..............",
        "gbdgggggdbhhhkhKeeeec.aL..............",
        "dddgggggbcfhhkhKeee....y..............",
        "..bggga.otfhhhhKeee...................",
        "...ddga..eEEohjHeee...................",
        "....bb...accEEHHeee...................",
        ".............ceeeee...................",
        "..............cceeeee.................",
        "..............ccceeeee................",
        "..............ccccceee................",
    ];
    pub const WITCHER_BACK1_ROWS: &[&str] = &[
        "............................PPPPP...........................",
        ".......................E...EOOOOOR...La.....................",
        ".......................yJ.tOOOOOOOO.yxc.....................",
        ".......................cCHOOOOOOOOKHxo......................",
        "...i..............ExxHctCDOOOOOKOOOHDxcECxyc................",
        "...b..............LHCCHCDxKOOOOKOOOHDDHHHHCc................",
        "...qq.ka..........cHCHHCDxHKDDDDDKODxHHCCxsa................",
        "...qqqqa...........oDCHCCCDDOOOOOCCCHHCCCoe.................",
        "iqqAIIvva...........CDCCCEPPPPPPPPPPCCDxs...................",
        "hqvAIIvvkd..Ec.......oHxCEPPPPPPPPPPCxxc........Le..........",
        "hAIINNAAAk..CP...yCyERRRCCCJJPPPPEECCPPREEHE...RJe..........",
        "hAIIINAIvnb.CORc.LHPPRRRRJCCCCCCCCDHPRRRRPHK..RRHe..........",
        "gAINNNNIvhaeDOPRRROCPRRRRRRRRRRRRPPPPRRRRJOPRRRPCje...Cx....",
        ".vAIINIvk..yCDOPRRPPJRRRRRRRRRRRRRRPRRRPJPPPPPECKHt.eCDDxj..",
        ".avvINvhb..cCxxJPPRRREPRRRRRRRRRRRPRRRJCLPPPPJxxCHj.EHHHHxo.",
        "..akkkkb.......yEHPRRPCHLRRPRRRRPPRRPECPPPPJKoo....sHHKHHHxc",
        "...aaa.......aabcJLRRRRCxJKOOPPPOOLCCPRRRPKyca.....sHHHHKHxc",
        ".............dggddjJPPPPPPxxxOKKxxxPPRPRJJedgd.....sHHKHHDxc",
        "....a........gggdddcJEEJPPPLOHHHPPPPRLLJtodgggb....oCHKHHCe.",
        "....jE......jgggddddbboEJLRRRPPPRPPLPCcfdgdgggy.....cxHCxj..",
        "....aJycc.ctCjgdddggddbdfEEPPPRPPLHycbddgggddgyo.....eooc...",
        "..JLccteoCxCDjgddgggdddddecECPPJCcebdbddgggddgyx............",
        ".EJEyyfeoDsDxjgddgggdddddddbcCCecbdddgddgggddgeDCEEe........",
        "..ECCJJJJHssxjgdggggdddggddggccbddddggddgggddgdCyffc........",
        "...........EyggdgggdddgggddggggggdddggddgggdddgoeffyJc......",
        "............cdddgggdddgggdggggggggddggddgggdddgofffyEEE.....",
        "...........bdddgggdddgggggggggggggddggddggggddddjoEEEEc.....",
        "...........ddddgggdddgggggggggggggddggdddgggddgd............",
        "..........ddddggggddgggggggggggggdddgggddgggdddgd...........",
        ".........ddbddgggdddgggggggggggggdddgggddggggddddb..........",
        ".........b..bdgggdddggggggdggggggdddgggddgggddddgda.........",
        "............bgggddddgggggddggggggdddgggddggggddddgdd........",
        "............ggggdddggggggddggggggggdggggddggggdddddda.......",
        "............ggggdddgggggggdggggggggdggggddggggga..bdb.......",
        "...........ggggdddgggggggddggggggggddgggddggggga............",
        "...........gggddddgggggggddggggggggddgggdddgggg.............",
        "..........dggddddgggggggdddgggggggggdggggddggggd............",
        ".........agddddddgggggggdddgggggggggdggggdddgggga...........",
        ".........ggddddddggggggdddggggggggggddggggddggggga..........",
        "......bdddddddddgggggggdddggggggggggddggggddddgggg..........",
        "......ddddddddddgggggggdddgggggggggggdgggggdddggggd.........",
        ".......ddbbdddggggggggddddgggggggggggddggggddddggggb........",
        "............ddggggggggdddggggggggggggddgggggdddgggggga......",
        ".............bggggggggdddggggggggggggdddggggddddgggggddd....",
        "..............ddggggggdddgggggggggggggddgggggddddgggggb.....",
        "..............Jcdgggggdddgggggggggggggddggggggdddggggb......",
        "..............JKEa..gddddggggggggggggggddgggggddddbb........",
        "..............Lc.....adddggggggggggggggddbfgggy.............",
        ".....................cfffggggggggggffeea.oooeot.............",
        "....................ceeeebdggggda.cceee..ottjjc.............",
        "...................ceeeeec........ceeeec.ccca...............",
        "...................acccccc........ceeeec....................",
        "..................................ceeeec....................",
        "..................................acccc.....................",
    ];
    pub const WITCHER_CAST1_ROWS: &[&str] = &[
        "............................RPP.............................",
        "......................yec..POOOPoaccJ.......................",
        "......................yCyoPOOOOOPtjJc.......................",
        ".....................EyxCxOHEECJOPCDcL......................",
        ".................JKJJoyDDDOJJELHOODxLsJJEE..............ll..",
        ".................yCyynyCxxKKLLLKOODHLnLECC...........d.adl..",
        "..................tnnnnEyxOKJJJJOOxJnnnkEo...........qbkka..",
        "..................yynnnnkCOJJJJJOKKnnnkEt...........fvvvvqa.",
        "...................tyohkkHOCxHKHCKJnnkLJy...........qAANAvqk",
        "............E......cxHLjLOxKojCHHOKoEHHL......E....vvINNIIvv",
        "....cy......EJo.EPRRKKOHKOPPxjCPOOHHOPOPJOo..PH...avAANINIIv",
        "...jCx......EPLjxOPPPJJCDCOPCjCPKHKKPPPPROHERPH..akqNAANNIIv",
        "..xxDxs...EKxJPLCHKPPLOHDDHOHeCPKHKOPPRRJCPRRtCOLcgkvINNNvvv",
        ".yDDOODx..CHxsPRKDHPPJCOCHHOPxJOKKOOPRRRJHPOPHPOE..kvINNIAva",
        "oDDKOOKDyccsxytKOHHKRPCCHnCOPKOPPPOPRRRRJELOCyyt....qqNIAvb.",
        "xDLRJKRLxo....aEojoJPRPCCtsKOOOOOORRRRLLohhLefda.....gkqqk..",
        "xHPRRRRPxs...dbbyJkeCRRCKKxxxDDxxERRRROLhhkhJcddg......a....",
        "xHOHHPCPxs....ddyHoLPRRCHseejjjjojCRRRLHEhhhCcbd............",
        "sDKOHOPxxt....acsHhfRRRCDsfcCHHchoxCRRKHhhkhCcyj......Le....",
        ".ExKOOxsc....ayoxHhhfJHCxocJeeeLtoxCCysthhheHcHHK...oJyc....",
        "..tCKKxcyo..oxHsDDEhhtosocCxHHHHysDxyyfffhjEHeoxDCCLhyfjJL..",
        "...jjje.EyycHHxsxxChhbbaexsxDHDDDHCobbbdbftDDcjxDDDJhhfJHEt.",
        ".........jyftDsxssHtbbaajDssHHDDHKKybbbbbfoDDeosCoCsttJJHEE.",
        ".......acfhfcCxxssHLabbaKKOxDDDDOLPLcabbbboHDejooEEEEEyyEE..",
        ".......EEEftEebbjtJabbaLHHKPPHDCPRPLxLabbbfsDefbba..........",
        "......yECEEtbbbbaaabbaLHjoKPPOOPRRPKetEabbaasobddb..........",
        "............abbabbbaaLCcffLOOCHHPROKfhyLoababbbdbd..........",
        ".............bdbbbbaPcJeffeLPODHPROKfkkoLyabbdbdbbb.........",
        "...........adddbbbaJyJoffffLPPKPRROKfknnLyabbdba.bb.........",
        "...........bddddbbEHjHfkhffEPRPRRROKfknnkyLabdb...a.........",
        "..........bbbdbdbtxtfJfkkhffPPPRRROJfhhknkLcdddb............",
        ".........bbbbbbbbyEffJfkkhhfPPCKOPOfhhhhnkLcbddb............",
        "........bbba..abayyfeLhnkhhfPRCKKOKfhhhhhkfLcbbbb...........",
        "...............byEefJfkkkhhfPPPPPOKfhhhhhkhLcbbbb...........",
        "..............bbyCfeKfkkkhhfPPCHKOKfffhhkkhjLabbbb..........",
        "..............bbyyffLekkhhhfPPDHKKJcffhhhhhhLcbbbb..........",
        "..............bJjfhfLfkkhhhcPPOPPOKaaffhhhhfLcbbbbdb........",
        ".............dbEjfffLfhhhhbbPPPRRRLcceeeeffcLabbbdbd........",
        "............bdbyjhhfLfhhhfdaLOJRRPtcceeeeeffcJabbdbdbba.....",
        "..........abddctfhoytefffffaLKHOOLccceeeefffcJcbbdddb.......",
        "........abddbbyfhhoKceeeejfaLOJPOLeceeeeffffeJcb............",
        "....baaabbdddbyehhoKceeeeeaaKOPROLceeeccffffeL..............",
        ".....bbbdbddbbyfffoJcceeefbaLOPRLLceecJoeefffJ..............",
        "......bdddddbbyfELoJceeeeja..PPLcccee.cELLcffJ..............",
        ".......bbbbdboeoffoKceeeefa...L...eee.....PLcJ..............",
        ".........bbbbofofhoKceeee.........eeee......LJ..............",
        ".............offfhoKceeee.........eeeec.....................",
        "..............offfsKceeec.........ccccca....................",
        "...............ofjxHceeec...................................",
        ".................tytceeec...................................",
        "....................ceeec...................................",
        "....................ceeeee..................................",
        "....................ceeeee..................................",
        ".....................ccce...................................",
    ];
    pub const WITCHER_WHIRL1_ROWS: &[&str] = &[
        "...............................................a......................",
        "................................ggiiiiiimrrrrrrr......................",
        "............................dgddd..iiiiiizzzzrrrrppa..................",
        ".......................................iimmmzzzzzzzzua................",
        "...........tssoj..........................immmzzzzzzzzra..............",
        ".........sxCHDDxsojc.........................immmzzFFFzrrrb...........",
        "........cxDKKOKxxxxe............................rrFFFzFFFFzma.........",
        ".......oDDPPOPODxxxe............................bmmzzFMMMMMzra........",
        ".......xDOPOPOPOxxxe..............................dmmFMQFMQMzm........",
        "......jDDOHHHPPODj..............PPPPc...............rFMQQQQQFr........",
        ".....cxDDxOHOPKDsc........Joc..JOOOORa.ccL..........mFMMFMQQMra.......",
        "....csxDDDDOOKHs..........ECE.JOOKKOOR.eJ...........mFMMMMMMFra.......",
        "....xxDDDxxxxDt..........texCEKOKJKOPPtCs...........irFFMQQMzm........",
        "...osxxxsecjjo.......LLJJyxsDDKOHJJJJOxxsJJJJJ........iFMQMrd.........",
        "...ssxssoj...........EEELnJyssHOHPLPJODxEnnyxJ........bzzzzma.........",
        "..jxsssja.............jnnnnnEsHOHKJJHOCCnnnnEo.........iiid...........",
        "..sxssec..............yyEnnnntHOHHHHHOCnnnnEya........................",
        ".ossojc..........j....ayxnnnntKKsHHKHOCnnntyc...jo....................",
        ".osjjj...........Ej.....CCttkCOxsyxCHsKstKKt...cJJ....................",
        "josjj............CPy.LRRRKHCyOKOCjtCOOKHKOPRRRJPPt....................",
        "joojc............HORJHPPPCHHHHOPHjsOPHHHKLPRPOKRPs....................",
        "jeje..........yLPKHPPPKJPPLKCHCPHjsOPHKOPPPRPKPPxOLE..............b...",
        "jcje..........aeECDHOOLKPPLKHHHOKjsKKKKOPPPPKKKOCCEa...............d..",
        "j.j.............cjeeLLePKOHCHxyLOOKOPPPPRRPLfjCCc......y...........b..",
        "j.j.........Lo.aaaRLsfhhtLPKConLOPOOPPPRRRPfkhjCjje...PJ...........bb.",
        "j......EJLccjPfhhyjsHjhhfcPHHLLKKKKKKKJRRREhkheDehhhjjJccLLJe.....dag.",
        "jc......yHJjhfffhhhjCjhhhfcLHOPJxDxxxxsPPyhkkkJDehhhfhfhJHJJ......gag.",
        ".j.......yJHteLLhhheHjhhhfffsDCeejooojeyCckkkhCeffjjLjhyEJt.......ddga",
        "..........eJJLJCytjjDHjhhfddtCsjfeHHCjetCehhhtCeeottyJJLEj.......dgbg.",
        "...............cayjeDHjhfgbbajEeJEsssCoyobdfftDeeo...............ddgg.",
        "................ayjeDKjfbbbbbacExxCCHDoxcbbdftCeeo..............bdggg.",
        "................adtsDCfdbbbabaExssDDDDHscbbbbjCjtj..............ggggd.",
        "................bdosKobbbbbaboDDssDDDDxDKabbbbyooc.............bkqgg..",
        "................ddtxjfbbbbbbcHHOKKDDDDOPKobbbbboob............gkvqkl..",
        "............cceefgdEebbbbbbboDDDKOODDHPOKKsbbbbbcd...........gghqqlb..",
        "..........ojECDDsfgdbbbbbbbctOjeLPPOORRPHjEobbb............igkkqqkg...",
        "..........oDDKODxxjbbbbbbbcEeKjfLPPKKKRPEhhLjab......gggggqqqvqqkl....",
        ".........jCHHPPOKDCobbbbbaEofLjhLPPOHHPPEhhtJeb....aqAAAvvvvvqqkka....",
        ".........eDHKRPKOHDsabbbaEyjLokhPPRRJJRPEhhhLeb...aqvIIIvvvvvvqkg.....",
        ".........eDHKHHHPHDDtbaayEjkLokhfPRRRRRPChhhLeba..kAINNNIIAvvvqi......",
        ".........eCHPPHPPHDDxjbbyekkPokhfLPRRRRPCfkkLebb..qNNINNNNAvqkqb......",
        ".........aoDOOHPHDDDDxtttjhLskkhfLPPHHPPjhknhJcbbaqNNNNNINIvqga.......",
        "...........jHHKHHDDDDDDxCyjLokhhfLPPHHEOehnnkLcbdbvININNIIAqga........",
        "...........coxDxxxHHDDDDDDCHxtttoHCPPPPJehkkkhLcdbqvvNNNIvqk..........",
        ".............eoxttxxDDDDDDDDxxxxsHCHCHPKCEkkkkLcdbbqvNNNAvg...........",
        "...............jofjeCDDDDDDDDDDxsHHHCCLOehhkkkLLab.bqvAvvha...........",
        ".............aggggdbcCxxxDDDDDDxxKORJERLchhkkheLaa..addgda............",
        "............bdgggbbbbCooooHxsssteJRRRRPLceeefffL......................",
        "..........ddggggbbbbayokhjHsejfaaJRRRRPLceeeeeeL......................",
        ".......dddgdggggbbbbEjhhfLeeeecbaJOOKJOceeeeeccL......................",
        "......adggdggggbbbbayfhhhKceeeabaJOKKJOceeeeccfL......................",
        "........dggggggbbbbaEfhhjKceeaabbJOKPPKeeecceffL......................",
        "..........ggggbbaa..EfhhoKeefabbaJOKRLOeeeLoccfJ......................",
        "............gda.....offkhLeecab...EKPLjceec.LLeL......................",
        "....................tfEthLeea.a....JLjeeeec...LJ......................",
        "....................tfLffJcc........J.ceeec...........................",
        "....................offEeEca..........ceeeea..........................",
        "....................eoheJHcc..........cceeeee.........................",
        ".....................cEECC.............cccece.........................",
        ".....................ejooj............................................",
        ".....................eeeeee...........................................",
        ".....................ceeeee...........................................",
        "......................cccc............................................",
    ];
    pub const WITCHER_WHIRL2_ROWS: &[&str] = &[
        ".................yttt..........giiiililllllli.........................",
        "...............exDKPKCc.............billmwwurlpli.....................",
        "..............jCDHKPLKxc..............ailllpwwwwpppb..................",
        ".............yxxHOORRPCs.................iilpppwwuppi.a...............",
        "............yxxxKOPRRRHs...................llllwwwwurrrib.............",
        "..........cCxxxxHOPRRPCs.....................illuwzzzMQFBa............",
        ".........cyxxxxDDDOPPLxo.......................lppzFMQQFFma...........",
        "........jsxxxxxDCDDOOxe..........................zFMQQMQMFd...........",
        ".......oxxxxxxxCejoooccc...tLOPP.................dFMQQMQQFi...........",
        "......tsxxsxsocc.....ayj...ROOOOP.ceP............dzMFQQzMFi...........",
        ".....tCxxssto.......aaoEe.PPOKKOPjcJC............brFMQQMFzb...........",
        "....jCxxxseca........PcCxxPLOKHLPJCHCc............arFQMMzpb...........",
        "...jCxxooj.......JJJLyysxCOKJJJJPKDxxEJEJL..........mrrrb.............",
        "...yxsssja.......JCEtnyxsHOKJPJJPKxxJnyECE..........bddi..............",
        "..txxooj.........cynnnnExHOKKHKKOKxEnnnnEo............................",
        "..ysoojc.........atEnnnnyCOKHHKKOKKnnnkLy................gi...........",
        ".ossoje......E....aCynkkkCKxHHHHOKKnnnJs...oJj.............iid........",
        ".oosoe......aKO...aeCJLfoCHCxxCDxKHoLKKc...PK................iii......",
        "ttott........JOocRRRPKKJyJOPKooCPOCHKKOPRLRRK......Gu.wi......dil.....",
        "toojo.aacc.acJPPPPPRRLHCHHPPLooCOHCKOPRRRPPPx....iFzFabup......ilid...",
        "tooaaLyeCHjHHCKJRKHPRPPOHHHPLtoHKHOOPPRRJCDCCJGdiMMMMdbiwa.....alii...",
        "toojcDDDDDDDDHHHPLJKPPLKHHHOOooHKHOOPPRPKCEsjebzFMQQMMzmra......dlii..",
        "totjosDHODDDDsxHehfJLPRHHCtKOOKOOOOPPPOLfhjHJaarMFMPGMFmzr.......illi.",
        "oosxKHHHPKHHHHHDjhkheLLHHokHKOKOPPPRRRLfkkkLjuBzFMRRQQQFFra.......lll.",
        ".jtxjKPPPPPOxssHjkkhfcJKKLLKKKKKOOPRRPchkkkJjlwzMQQQQQQFrr........llli",
        ".jsxjKPPPPPLEnkExhkhfffyHPOHCDDDDCKKLthhkkJJjkwBMQQQQQMzzra.......llli",
        ".txDHDDOPPPOxoheCjhhfffctJHeoooooEjCJfhhkkHjfhkpmQQQQMFmFb.......lpuli",
        ".jxDDsDOPOxxHxjjCehhfgbbtsxojoCHJjfyEcdhhkHjjjgFzFMMGFzrb........ipwpd",
        "..yCssDHHDxDsxseDjffddbbgotoEEoooLjCoabdhhCjjgwrzFFFFFrGb.......bipplb",
        "...xoDHDxDKKjxssDjfdbbbbbaftxxKHKHsycbbbdfCxt.aBzFFddFFza.......pplwl.",
        "...jsHseceocfyoxCfdbbbbbbJyxsxHHHDxCcbbbbboxj..aaaa..aa........iuuBw..",
        "....osEfggffdcoxjbbbbbbacCxDxxDDDDDHJabbbbaye.................ilpwwl..",
        ".....ssebggggdtoabbbbbbaLHOKKxDDDDHPKjbbbbbfc................lpwwwpl..",
        "......otcbgdgdofbbbbbbaEHCJKPPPDDHPLKJabbbbbb...............pluwwula..",
        "........ocabgddggigbaayDOefLPPPOOPRPojLcbbbbbba..........luplBwwull...",
        "........oyabbgkvINIvgooHehhLOOOHHHRPohLcbbd..ba....ipupdluwwBBuuup....",
        ".........ggggqNNNNIIvEeJfkhoPPPHHHRPoktLcbb..b....bBMQMMzwuuuuupg.....",
        "........ddgggvNNNNNNIhLEhhhfLPRPHRRPokhLedd......dBzQQFFFzzuuuli......",
        "........dggggvNNNNNNIIAkkhhhfLPRRRRPokhhLtdb.....zMQQMMMMFzzul........",
        "........adddgvNNNNNNIAAvqkkhfJPRRRRPokhhEtdd....iBQQQGMQFFzwu.........",
        "..........bggvqINIINIAAAAAAkhJLPHKRLjhkhJobdb...iBQFQQQMFMzu..........",
        "...........ggggqINIIAAAAAAAvqALLJJLLjhkhhLodb...awMQQQQFMmb...........",
        "..........bggggggAqkkAAAAAAAAAAAANRNhkkkhJjddd...aGrQQMQma............",
        ".........agggggddEthLkqvAAAAAAIIAAAAIIAAvJJfdda...drBMzmi.............",
        "........ddggggdbaEkhJjkkqqvvvAIAIIIAAvkkqkPjdddd...diida..............",
        "......ddgggggdbbyfhfJfhkkkkggvPPRPPPokkkkkJjdddg......................",
        "....dddggdgggbbbtfhJjhhhhhhbatPRRRPPcfffhfJjbdd.......................",
        "..dggggggggdgbbayffJjeffffgabtPRRRLRceeeeeJt..........................",
        "...bdggggggdbbateffJjeeeefbabtKPKPKoeeeeeeKt..........................",
        "....ddggggdbbbbtfhfKjeeeeaabboHOHPKoeeeeeeKt..........................",
        "......dddbbbbbbtefjKjeeecaaaaoHOPPJoeeceeeKy..........................",
        ".........bbbbbbyfLEKjeecaaa..oOLRRHoeeLeeeKy..........................",
        ".............atftffJjee........LRRoeec.PLoK...........................",
        "..............oftffJjee.........Ljceec...EJ...........................",
        "..............offjjJee...........cceec................................",
        "...............jfoyCee...........ceeeea...............................",
        "...............cofeCe............cceeeee..............................",
        ".................tExe.............cccccc..............................",
        ".................eeje.................................................",
        ".................eeeec................................................",
        "................aeeeee................................................",
        ".................ccee.................................................",
    ];
    pub const WITCHER_THROW_ROWS: &[&str] = &[
        "..........................................POOOc............................",
        "....................................yoa..jOOOOL.aaEc.......................",
        "....................................Jyf.jOOKOOOLojye.......................",
        ".....................................JCcCOOKKOOOtEc........................",
        "...............................ccccEteJDCOCEJJJOCxcyyyCy........qqa.qqa....",
        "...............................yEJEnqEsxDOEJJJOOHxEyoECy......a.aa..bqb....",
        "...............................connnnyEsDOPLJJLOHCnnnkEj.....qqaa....bqa...",
        "................................eynnnntxDOJJJJJOHsnnnEE.......bvaagg..a....",
        ".................................yonnkksDOKJJJHKDknnnHo........aaaAIk.ad...",
        "..........................E......cHCykktHDjKHHxHCnnEHCj...LE...dhfIIAbabda.",
        "..........................Kt...tyEHKOLjPKDsojCHKKCCKOKKy.RKy.avAAIINIvgbba.",
        "..........................JKLjELPPLPKHKHOOHstJOHDHKPPPPOPOKjabkvIIANNIvbbba",
        "..........................CHPPLHPRPPPCDHCPHstKOHKOPRRRRPHPHOJIvIIAANNNIgbba",
        "........................EJDxCPLHPPRLJJHHHOHstHKHHOPRRRRJOOHHJovIvANNNNIgdda",
        ".........................cCCxOHHHOPLHKKHCOKOOKOOOOPRRREoJPofhhvvANNNNIvvda.",
        "..........................dfftssoJPPOCEooLPOOPPPPPPRRohoCehhhjJeIINNNIkggb.",
        ".........................gggggytjftLPKHyxKKKKOOKPRRRNhkoCehkkJLcgINIAAga...",
        ".......................ggggghLCCohhjPOLPCCDDDDDDKRGyhkkoHehfotaafvvvqbaa...",
        "......................igghhEHHoyCthhfCLPjjjjjjjjxPfhhhkoHeobabdAvbaaaalq...",
        ".....................dgyyoLHDfhhxshhfoxsjftCCEjhyHegbhkoHeoe...gb..aqvaa...",
        "...................ddhhkEEDDDDohCDfhbbxxooCsssyjyx.abbfkHcCc........bda....",
        ".................dggEqfyEsKKKHsfCDegbbajoxxCCHDxte...bbbJoCc...............",
        ".............dggggggJfgCsLPLOKxDDCebbbajCxxDDHDDsc.......Lo................",
        "dd......gggggdgggggghggECPLLLODsDebbaajCxsxDDHDDCJ.......a.................",
        "dgggggggggggggggggggghgfEPPPPODHDcbbaaKHOHDHDDDKPPOc.......................",
        ".ggggggggggggggggggbbyofyCPLOPCDDcbbayHHOOOOHDKOPLCJ.......................",
        ".dgggggggggggggggbbbbcJcJDDxHjCsjaaacCxsLOPPPOPRRLeLy......................",
        ".bgggggggggggggbdbbbbbbbayCxEfjjbbboyEefoLOPOOPPRKffL......................",
        "..agggggggggddbbbbbbbbdddfjyfEybbbtjytfhhPPPHCCORJehfJ.....................",
        "...bgggggdbbbbbbbbbbdddGgdtbbaaaatoLCohhhERPOCDKRJfhhJL....................",
        ".....bbbbbbbbbbbbbddggddbbbabaaaojjKyhhhhfRPPKHRRJfhhfyE...................",
        ".......bbbbbbbbbddddgggdbbbbbbaojfjLEkhhhffPPPRRRJfhhhfCe..................",
        "......bbbbbbbbddddggdddbbbbbbbtffhEefkkhhffLPJHKPLfhhhhEc..................",
        ".......bdbbbddddgdddddbbbabbbothhyehhkhhhffLPKHHPLfhhhhhJ..................",
        ".......bdbggddgggdddddbbbabbjthhftjhkkhhhfbLPPRRPKehhhhhfLj................",
        ".......adddgdggggdddbbbbbbbatohheHjhkhhhhfbLPPPRPKehhhhhhJe................",
        "........ddgdddddggdbbbbbbatyjfhyHthkkhhhhbaLPOHPRJcffhhhhjEc...............",
        "........ddggggdddddbbbbbbayofhfLthknnhhhfbaLPPHHPJ.bffhfheJc...............",
        "........bdggggdgdbbbbbbdbhefhhojfhhhkhhhdbaLRPLOPJ..beeeeeJe...............",
        ".........bdggggdbbbbbdbbofhhhjJeffhhhhhfbbbLRRRRRJ..ceeeeeJe...............",
        "..........bddbbbbbbbbbooyffffEJcceefhhfbbbbJPPJPLJ..ceeeeeJe...............",
        "............bbbbbbbbbbofffhhtKecceeeffdabbbEOOHOLL..ceceeeJe...............",
        ".............bbbbbbbbbofffEfCKcecceeefaa...JOOELJ...ceeeeeJe...............",
        "...............bbbbbbdtfyofjJyceeeeeee.....EOPRLL...ceeeeeJe...............",
        "................bbbaaaoyeyfLKceeeeeea......yOORLc...eeeecfJe...............",
        "......................ctjteL.ceeeee.........cKRLa...ceeec.Ee...............",
        ".......................oeeJJ.ceeee............P.....eeeec..................",
        "........................tcJ..ceee...................eeeeec.................",
        ".........................EE..cee....................ceeeeeea...............",
        "............................eeee....................ccccccec...............",
        "............................ceee...........................................",
        "............................eeeee..........................................",
        "............................eeeee..........................................",
        "............................cceec..........................................",
    ];
    pub const WITCHER_BURST_ROWS: &[&str] = &[
        "................................................aaaaaa............",
        "................................................abbbba............",
        "................................................abbdba............",
        "..............................................Gdadbggb......Rg....",
        "..............................................Pjadfllg.....R......",
        ".....................a...JOOP.................Pjagkqqida..........",
        "....................yca.POOOOP...oy..ga....aaaaaboqvvqgba.....ia..",
        "....................EoaRPOKKOOP.oJo.aabaaaaabbddgEEPJvgdbaa..Eo...",
        "...................JaysOPEEEJJPjKDJ....EjbddgilqqLLPqqigddbaEt....",
        "...............JKJLtycxOPJJJJEOHDxxLLJEyLLglppqAyPOLvqliigcEa.....",
        "...............EsEnnJEsOPJPOPJPHsxJnvtCCtJEqvvALLPPLAAvqqfLoa.....",
        "...............ctnnnnEtOOJJJJJPHxEnnnEEjgELPRAIPPRRPPLLLLLEba.....",
        "................jJnnnntOOJHHHHPHxnnnnyydlqRPPLLPRRRPPLLOOJqba.....",
        ".........dc......ttkkkjHOxHEEHOHsnnoytjglqqPOPPRRRRRPPPPJvqbaaRR..",
        ".........cLc.....cCHCeeHHLjooHCOOoCKKjigPyvyPRRRRRRRRRPLLvqgba....",
        ".........cOPo.oPPRRPHCKCOOCjoKPPKLKOOPPRPEEEPRRRRRRRRRPPPLLidaa...",
        ".........eKPPtCKPRPJECDDHOHjoKPKHOLPPPPOOOPRRRRRRPRLRRRPPEqldba...",
        ".......tJPJJJLCCKPPLLCKDHPHjsKOHOPPRRRPHKKORRyNPPRRRRRRRPRLkhqqqqb",
        ".......cLJHxEPPCCPPLKJDKDOOKHKOOOPPRRRKKKKOPPqAPPRRRRRRPPPPRLEbba.",
        "..........cacJKjoKPPCCEtxKOKKOPPPPRRRRLoOJooJqvLPRRRRRRPLGAlgbba..",
        "...........bbeehkoPPRxCosHOOOPPPRRRRREtkLokEEhqLRRRRRRPPJAIldba...",
        "..........bbbdfkkhfPRxHKKHHHHHHHRRRRRtkLLoqvqRRRRRRRRRPPLAAidba...",
        "...........agffhhhhoPxHKxjooojojKRPRJhqOKokqtPRRRRRRPREPRqkidaa...",
        "...........ggfECyEkhjxDsjhjCCHCjoJJofkkLKokPPOPPRRRPPPAvqqgba.....",
        "...........gfJeffyEhfxxsjheHHCohjCCegkkkKjtPPPPLPRRPPLLPqidRLga...",
        ".........abdECefyJEodcyyosCeeeKtCsxedgkkJjJPPRRILPPENLqyLodoLg....",
        ".........bbcJxeLJJJjbbacjxxCHHDxCeebddghLLLRLyvvvLLvvvlgtLLda.....",
        ".......abbbbKxeLJJJjbbaECsxDHDDDDHjddddgoLjgEkqqgqEkligbajLba.....",
        "......abbdbbJsefyJEcbajCHHxDxDDHCKycddddgJjgygligiiggddbaaau......",
        ".......bbdbbatcjjyjbaoCHKKOHHDDPRPHoddddgddggbdbbbbbbbbaa.........",
        "........bbbbaycfEcbbjCDHKKPPKCHRRPKCEddddggdgbaaaaaaaaa...........",
        ".........bddatyeycbaxHyeeOPRPOPRRRjyCfddddddgbbaaaaaRya...........",
        ".........dddbaayocctHHeffJPREHCJRPjyJjddbddbgqdaa.................",
        ".......addddbbbcacELEohhhyPRPHHHRPthoLodddaagla...................",
        ".......ddddddbbbatejEfkhhfPRRHHRRPyhhLEjddaawb....................",
        "......dddddddbbeEyfjykkhkhfLPRRRRPjkkkJxdddb......................",
        ".....bbbaadddbbEtfhjykkhkhfLPRRRRPjkkkJJfddb......................",
        "...........dddcJshLtkkkhhhfLLHCPRPjkkkkJeddbb.....................",
        "...........dddJehhLokkhhhhfLPJHKRPjhkkkkLoddb.....................",
        "..........abdbEehyLokkhhhhfJLPPRRPefhkkkJEabdb....................",
        "..........dbddEehKskkkhhhffJPOPPRPefhhhkoJcbddb...................",
        ".........abbdEefhKskkhhhhffJPJHHPKcbfhhhhJcbddbb..................",
        "........abbbeCfhfKohkkhhhbbJPRKHPKcacfeffJcbbdbbb.................",
        ".......abbbbetfffJohhhhhfbaJPRRROJccceeefjEbbdddbb................",
        ".......bbbbbjofhfJofffhhfbaELJRRKecceeeeffJcbbbbb.................",
        "....abbbddbtefhfLjcceeffbbaEOKJPKjbceeeeffJcbbbb..................",
        "bbbddbbbddbtefhfKjeeeeecbbaEKKOOOjbceeefffJ.......................",
        ".abbbddbbdbteffeHjeeeejbbaaEKOPOKobcceeeffJ.......................",
        "..bbbbbbbbbteyeJHjeeeeebb..EOOROKcaceeeefeE.......................",
        "....abbbbboeyojHHjeeeecba...cLPJc..eeeeyLcJ.......................",
        ".....bbbbbocyehKHoeeec.......cLc...eeeec.JJ.......................",
        "..........offejJjeeee.........a...aeeeec..........................",
        "..........oofjeJjeeec.............aeeeeec.........................",
        "...........ooetEjeee..............aceeeeeea.......................",
        "............ctyceeee...............ccccccca.......................",
        "..............caceee..............................................",
        "...............aceeee.............................................",
        "...............ccceee.............................................",
        "................cccc..............................................",
    ];
    pub const WITCHER_AFTER_ROWS: &[&str] = &[
        "...........................ce...............................",
        "...........................EC...............................",
        "............................................................",
        "......tEy..................................aGg..............",
        "......eyy..................................gIa..............",
        ".......cac.....tHc..........................................",
        "..........e..ccHDc..........................................",
        "..........eycjxHDcEaaj.....PPPP........................qa...",
        "...........tHeHHDCxcaJo...POOOOP...Et..............ia.db....",
        "............coKKKHDccHHjaRPOOOOOOestj..............lbvqq....",
        "...........oCDOKOKHHeCDHyPOHOKOOOsxjc..............qvvvva...",
        "...cJ.....yDDHPRRPHHDHDCHLOEEJEHOHDyJPLJE.........qvINNvvb..",
        "....o.....yDHOPRRPPKHHDCELOKPJPOODDynnExC........gkvINNAvqb.",
        "..........CDHRPRRRRLHHxyyLKHKJJKOHLnnnnEy.......aqIIININIvva",
        "..........EDKRRRRRRKHHnyCKKHKKKHPJnnnnyC........aqININNINvqb",
        "..........EDHPRRQRPKHEnyCKCxKKHKKJnnnsCy....yc...qAINNNNIvqa",
        "..........jHHKPRRROHDCyJHCCoEHCyCKoJCHOa...EJc...bkIINNIvqk.",
        "..........cJKKKPPKKKKKCHHOPCoLPOHHKHKOPPL..POc....avANNAvd..",
        "..........cLPPKKKHOPLKHHHOPCoJPOHHKKLPRRJOPRLc.....avqvvq...",
        ".......ycJOHKRPKHKPPPPOKHHOCoKOKHHOPPRRJPPPJxKJE....dgga....",
        ".........EKHHJJHHCPPPPPHKHPOxKKKKKPPRROKPPLCCKO.............",
        "..........esCHJHxhyLPLOKxxOOOOOOPPPRRRLeeLcjc...............",
        "............eJEHLkkyRPPHssKOOOOOPRRRRPjhhJCob....L..........",
        "............eJJHHokhoRPHDHDCDDHDJRRRRLjhjHDofhffoecfEEE.....",
        "............JJJHstEhkPLDxejssssssPRRPtkhJDKfhfeffffhJJc.....",
        "...........dExeJoeCohhExxeessssjjyKOokhkHxeffeCeffJJEc......",
        "...........bEKyefcCjkfcxsecCHHCejxDjfhkoHDeftEsEEEyyc.......",
        "..........ddoHjffJDjhfbjotCejjeHjtEbffhjCDejCcc.............",
        "........abdddEjffHDebbbatxxHHHHDCxcbbbbfCDejy...............",
        "......abddddbfyefJobbbbJHssDDDHDDCabbbbbtCeoc...............",
        ".......bddbdbbyjfJcbbbcEHKsDDDDDDKeabbbdbtjtc...............",
        ".........bdddbtxjJabbaLHKKOOHDDHOLtygddbbctcd........lk.....",
        ".........bbddbboscabcJHHKKOPODDCPLcttdbbbbabb........kqa....",
        ".........bdbbbbaaaacCDEefLOPPOOPRLfhyybbbbbbba.......a......",
        "........bbbbbbbaaaexjKeffJOPEHJJRLfkoJobbbbdbb..............",
        ".......bdddbbbbbbyttLthhfePPPHHJRLfkkJebbbbaab..............",
        "......bdddbbbdbbeyffLjhhffPPRJJRRLfkkoLcbbb.................",
        "....bddddabbdbbbtofoLfkkhhfPRRRRRLfhhhhEcbbba...............",
        "....aa....bbbbdEjhfLthnkhhfPRRRRRLehkkkEcbdda...............",
        "..........bdbbdEjhfLfknhkhfPPHHPPJehkkkhLbddb...............",
        "..........dddbyjhhJehnkkkhfPPHHHPJcfhkkhJcbbb...............",
        ".........bddbbEoheJehnkhhhfPRPPRPJcffhkkhLadbb..............",
        ".........bddbytfhLohhkhhhhfPPPPRPKcfffhkhLcbbb..............",
        ".........bddjEohhLehkhhhhfdPPJHRPJabfffkhLcbbbd.............",
        ".......abbddEofhfPehkhhhhbaLPJJEPJaaceffeLabbbda............",
        "......abbbdbEohhLcehhhhhfaaLRRRRLJaaeeeeecLobbdbaa..........",
        ".....bdbdbbtcfhfKceefhhfaaaJPLLPKaaeeeeeefEobbbbb...........",
        "....bddddbbtehfKKceeeefbbbaJKKKLKaaceeeeefJoa...............",
        "bbddddddbdocfffJeceeeeebbbaJKJJOKaaceeeeefLo................",
        ".bdddddddboefEjJeceeeeebbbaLORPJHabceeeeffJo................",
        "..bdddddboofEoooeceeeecbbbaLOPPKJ.aceeeeefJo................",
        "...bbbbbbtehffJeceeeee.a...ayKPL...aceeeJeJo................",
        ".....bbbbtehhfLeceeee........jJc...aeeec.EEo................",
        ".........cyfEyKeceee...............ceeec....................",
        "..........tfjHecceee...............ceeeee...................",
        "...........EJx.ccee................ceeeeeea.................",
        "...............ceee.................ccccccc.................",
        "...............ceee.........................................",
        "...............ceeee........................................",
        "...............ceeee........................................",
        "...............ccccc........................................",
    ];
    pub const WITCHER_GHOST1_ROWS: &[&str] = &[
        "..........................dwBBBa...dd...............",
        "......................wp..wBwwBGb..pi...............",
        "......................uulBBBuwBBubuwi...............",
        "....................lpuuwBuuuuuwBwwwi...............",
        "................uBBBwwuuuBuBBuBwBuuuwBwwBa..........",
        "................bwwuuuwuuBuuuuuwBuwwwpupu...........",
        ".................iuupuuwuBuwwwwwBwwuppppb...........",
        "..................uwpuuuuBuwwwwwBwuuppud............",
        "...........lu......wwupuuBuwwwwwwBupuwub....bw......",
        "...........lBG..a.wwwwBuwBwwuuuwwBuwBBwd....pw......",
        "...........lwBG.bBBBBwwuwBBBppwBBBwwwwwBwG.uwu......",
        "...........pBwBBBBBBBwwuwuBBuuBBwwwwBBwBBBBwBw......",
        ".........wBwBwBBBuuBBwBwwuBBupBBwwBBBBBBBBBwBBBu....",
        "..........uwwwwwwwwwBBBwwuwBuwwwwBBBwBBwwwwuuuw.....",
        "............bbduuuuuwBBwwppBBBBBBBBBBBBuuupdbdb.....",
        "...........bdbwwwwwpuBBwwuwBBBBBBBBBBBwpuwui........",
        "..........gpppwwwwwupBBBBwwwwwwwwwBBBuupwBwi........",
        "...........aluuBwwuuuuBBBwuuuuuuuuBBBpuuwBww....bg..",
        "............wwwwwwuuppwuuuupwwBuuuuwpppuwBwuBBwwBwu.",
        "...........uuBuwuBuuppuuwuuuuppwuuuupppwwBpuuuuwBwwu",
        "..........puuBppuBuuppllluuuwBwwwppppppuuwpppuuuwwwu",
        ".........puuuBuupwuppllllpuuwwwwwwuppppuwwpBpwuwwwwa",
        "......aluuuuuBuupBuplllpwwuuwwuwwBBwpppuuwuBpwbaa...",
        ".......aupppllwpBwllllluBwwwuuuuwwBBppppuBuBpa......",
        ".........pppllupBwllllwwwwwBwuuuBBBBBppppBwBppl.....",
        ".........pppllpBwplllwwupBwBBwwwBBBwwBlppBupppp.....",
        "........lpppplllllllwBpppBwwwwwBBBBwwBBppppppppl....",
        "........pppppllllllwBBppppBwwuuuBBBupwBpppplllpp....",
        ".......pppppppllllwwwBpuppBBBwwuwBBuupwBllll...l....",
        "......lpppppppllllwuBuuuuppBBBBBBBBuuuwBwlll...a....",
        ".....lppdbdppppllwBuBuuuuupuBBBBBBwuuuwwBlppla......",
        "..........dpppppwBuuBuuuuuupBBwwBBwuuuwwBpppp.......",
        "...........pppppBppuwpuuuuupBBwuBBwuuuuuwBppll......",
        "...........ppppwBpuBuuuuuuupBBBBBBwuuuuuwBppplb.....",
        "..........pppppBuuupppuuuuppBBBBBBwuuuuwwBBlllld....",
        ".........apppppBuuBuppuuuuplBBwwBBwuuuuuuwwlllll....",
        ".........lpupuBuuuBpuuuuuuplBBBBBBwpppuuwwBllllli...",
        "........lppppwBuuuBpuuuuupplBBBBBBBullllpuwllllllli.",
        ".......ppulppBuuuuBpuuuuupllBBBBBBwuplllupwBllllllpl",
        ".....blpplllwBuuuwBplpuuuullBBBBBBwplllppppBlplllld.",
        "aggippppppllwwppuwplllluplllBBwBBBllllpppppBi.......",
        ".lppupplppllwpppuBpllppuplllBBBBBBllllpppppB........",
        "..pppppplpppuuwwuBplppuuplllBBBBBBlllppppppB........",
        "...llpplllpuuuuuuBplpuuuilliwwwBBBllllwupppB........",
        "....bilpllpBuwuuBwlpuuudbl...wwBiipppbddBBpB........",
        "......iiiipwuwuuBwlpuui.......walipppb...aGG........",
        "..........blupBwBulpuu..........llpppp..............",
        "...........aBBpwwllpuu..........llppppli............",
        "..............uwwllpup...........ilppppla...........",
        "................adlppp............iiiigg............",
        ".................llupui.............................",
        ".................lluuul.............................",
        ".................lllppl.............................",
        "..................llll..............................",
    ];

    /// A little fire spirit he calls up, two flickers of its flames. 6 yellow, 7 orange, 8 red,
    /// a dark (eyes, mouth).
    pub const WITCHER_SPIRIT: [&[&str]; 2] = [
        &[
            "...7.....7..",
            "..787...787.",
            "..7887.7887.",
            "..78866688..",
            ".7866666687.",
            ".786a66a687.",
            ".7866666687.",
            "..7866a687..",
            "8.78666687.8",
            "78.786687.87",
            ".8..7887..8.",
            "...78..87...",
            "..78....87..",
            "..88....88..",
        ],
        &[
            "....7...7...",
            "...787.787..",
            "..78877887..",
            "..78866688..",
            ".7866666687.",
            ".786a66a687.",
            ".7866666687.",
            "..7866a687..",
            ".8786666878.",
            "7..786687..7",
            "....7887....",
            "...78..87...",
            "..78....87..",
            "..88....88..",
        ],
    ];
}

/// A premium hero pet with the usual defaults (paw "w", eyes at columns 9 and 12).
#[allow(clippy::too_many_arguments)]
fn hero(
    id: &'static str,
    name: &'static str,
    palette: &[(char, Color)],
    body: [&'static str; 13],
    feet: [&'static str; 2],
    sit_base: Option<&'static str>,
    tricks: &'static [Trick],
    typing: TypingStyle,
) -> Pet {
    Pet {
        id,
        name,
        palette: Palette::new(palette),
        body,
        feet,
        sit_base,
        paw: b'w',
        eye_cols: [9, 12],
        tricks,
        typing,
        sweeps: false,
        reads_with_glasses: false,
        premium: true,
        wind_key: false,
        floats: false,
        picture: None,
        pastime: hero_pastime(id),
        trick_gap: crate::pet::TRICK_GAP,
        favorite_trick: None,
        turn_trick: None,
        wake_trick: None,
        drinks: true,
        click_trick: None,
        right_click_trick: None,
    }
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
            out.push(if swish { format!(".{}", &s[..s.len() - 1]) } else { s.to_string() });
        }
        out
    }

    /// An open book held up in front.
    pub fn book(flip: bool) -> Rows {
        rows(&[".kk.kk.", if flip { "kiiksik" } else { "kiikiik" }, "kiikiik", "kiikiik", "krrrrrk"])
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Trick {
    Firework, Flashlight, FlipEgg, Campfire,               // Ember
    HideInWater, SpitWater, Swim, Bubbles,                 // Pip
    IceCream, Twin, ColorShift, PopFood,                   // Puddle
    GiveFlower, Shower, PlantSelf, GrowUp, ChaseButterfly, // Sprout
    Yarn, BoxNap, Laser, Fishing,                          // Mochi
    GrumboSpin, GrumboHelmet, GrumboChop,                  // Grumbo
    WispThread, WispKnot, WispMoth,                        // Wisp-a-ling
    HexFrog, HexSheep, HexBook,                            // Hexley
    MossToss, MossBloom, MossRain,                         // Mossback
    ClankWind, ClankSpring, ClankBell,                     // Sir Clank
    LunetteRice, LunetteWish, LunetteNap,                  // Lunette
    NinjaSmoke, NinjaShuriken, NinjaHide,                  // Kage
    NinjaClones, NinjaFlip, NinjaMeditate,
    SamuraiDraw, SamuraiBow, SamuraiBamboo,                // Ronin
    SamuraiTea, SamuraiSpin, SamuraiFly,
    MoonaPounce, MoonaGlaive, MoonaTriplicate, MoonaMoonlight, // Moona
    WitcherOrb, WitcherWhirl, WitcherWeigh, WitcherVanish,     // Orbis
    WitcherSunStrike, WitcherTornado, WitcherMeteor, WitcherMoonBlast, WitcherSpirits,
}

/// One throw of Moona's glaive, out and back; clicks during a throw add more throws.
pub const GLAIVE_THROW: f64 = 2.0;

impl Trick {
    pub fn duration(self) -> f64 {
        use Trick::*;
        match self {
            Firework => 8.0,
            Flashlight => 8.0,
            FlipEgg => 9.0,
            Campfire => 10.0,
            HideInWater => 9.5,
            SpitWater => 6.0,
            Swim => 10.0,
            Bubbles => 7.0,
            IceCream => 9.0,
            Twin => 11.0,
            ColorShift => 8.0,
            PopFood => 7.0,
            GiveFlower => 7.0,
            Shower => 9.0,
            PlantSelf => 10.0,
            GrowUp => 9.0,
            ChaseButterfly => 12.0,
            Yarn => 10.0,
            BoxNap => 10.0,
            Laser => 10.0,
            Fishing => 11.0,
            GrumboSpin | GrumboHelmet | GrumboChop | WispThread | WispKnot | WispMoth | HexFrog | HexSheep | HexBook
            | MossToss | MossBloom | MossRain | ClankWind | ClankSpring | ClankBell | LunetteRice | LunetteWish
            | LunetteNap | NinjaSmoke | NinjaShuriken | NinjaHide | SamuraiDraw | SamuraiBow | SamuraiBamboo
            | NinjaClones | NinjaFlip | NinjaMeditate | SamuraiTea | SamuraiSpin | SamuraiFly => 8.0,
            MoonaPounce => 2.6,
            MoonaGlaive => GLAIVE_THROW,
            MoonaTriplicate => 5.0,
            MoonaMoonlight => 4.4,
            WitcherOrb => 1.8,
            WitcherWhirl => 3.2,
            WitcherWeigh => 3.8,
            WitcherVanish => 4.0,
            WitcherSunStrike => 2.6,
            WitcherTornado => 3.7,
            WitcherMeteor => 5.6,
            WitcherMoonBlast => 2.6,
            WitcherSpirits => 8.0,
        }
    }

    pub fn title(self) -> &'static str {
        use Trick::*;
        match self {
            Firework => "Light Fireworks",
            Flashlight => "Explore with a Flashlight",
            FlipEgg => "Flip an Egg",
            Campfire => "Toast a Marshmallow",
            HideInWater => "Hide in the Water",
            SpitWater => "Spit Water",
            Swim => "Go for a Swim",
            Bubbles => "Blow Bubbles",
            IceCream => "Eat Ice Cream",
            Twin => "Split in Two and Play",
            ColorShift => "Change Colour",
            PopFood => "Pop Out a Snack",
            GiveFlower => "Give You a Flower",
            Shower => "Take a Shower",
            PlantSelf => "Plant Itself",
            GrowUp => "Grow Up",
            ChaseButterfly => "Chase a Butterfly",
            Yarn => "Play with Yarn",
            BoxNap => "Nap in a Box",
            Laser => "Chase a Laser Dot",
            Fishing => "Go Fishing",
            GrumboSpin => "Spin Until Dizzy",
            GrumboHelmet => "Helmet Slips Down",
            GrumboChop => "Chop a Log (Try To)",
            WispThread => "Reach for the Pointer",
            WispKnot => "Get Tied in a Knot",
            WispMoth => "Attract a Moth",
            HexFrog => "Turn into a Frog",
            HexSheep => "Turn into a Sheep",
            HexBook => "Read a Biting Book",
            MossToss => "Toss a Rock, Then Nap",
            MossBloom => "Sprout a Flower",
            MossRain => "Enjoy a Little Rain",
            ClankWind => "Run Down and Wind Up",
            ClankSpring => "Sneeze a Spring",
            ClankBell => "Tea-Time Alarm",
            LunetteRice => "Pound Rice Cakes",
            LunetteWish => "Wish on a Shooting Star",
            LunetteNap => "Nap in a Crescent Moon",
            NinjaSmoke => "Smoke Bomb",
            NinjaShuriken => "Throw Shuriken",
            NinjaHide => "Hide in a Bush",
            SamuraiDraw => "Quick Draw",
            SamuraiBow => "Bow",
            SamuraiBamboo => "Cut the Bamboo",
            NinjaClones => "Clone Trick",
            NinjaFlip => "Backflips",
            NinjaMeditate => "Meditate",
            SamuraiTea => "Tea Ceremony",
            SamuraiSpin => "Whirlwind Slash",
            SamuraiFly => "Catch a Fly",
            MoonaPounce => "Pounce",
            MoonaGlaive => "Throw the Glaive",
            MoonaTriplicate => "Triplicate",
            MoonaMoonlight => "Call the Moonlight",
            WitcherOrb => "Cast an Orb",
            WitcherWhirl => "Whirl the Orbs",
            WitcherWeigh => "Weigh the Orbs",
            WitcherVanish => "Vanish",
            WitcherSunStrike => "Sun Strike",
            WitcherTornado => "Tornado",
            WitcherMeteor => "Meteor",
            WitcherMoonBlast => "Moon Blast",
            WitcherSpirits => "Call the Fire Spirits",
        }
    }
}

/// Pixel art for tricks, facing right, coloured from `prop_palette`.
pub mod trick_art {
    use super::{rows, Rows};

    pub const ROCKET: &[&str] = &[".k.", "krk", "krk", "kyk", ".n."];
    pub const FLASHLIGHT: &[&str] = &["kkkkk", "kgggy", "kkkkk"];
    pub const PAN: &[&str] = &["....kkkkkk", "nnnnkddddk"];
    pub const EGG_UP: &[&str] = &[".iyi.", "iiiii"];
    pub const EGG_DOWN: &[&str] = &[".iii.", "iiiii"];
    pub const FIRE_A: &[&str] = &["...y...", "..yoy..", ".yooy..", ".yoooy.", "nknnnkn"];
    pub const FIRE_B: &[&str] = &["....y..", "..yoy..", "..yooy.", ".yoooy.", "nknnnkn"];

    pub fn stick(toasted: bool) -> Rows {
        if toasted {
            rows(&["....yy", "...nyy", "..n...", ".n....", "n....."])
        } else {
            rows(&["....ii", "...nii", "..n...", ".n....", "n....."])
        }
    }

    pub fn pool(width: usize) -> Rows {
        let w = width.max(2);
        vec![format!(".{}.", "C".repeat(w - 2)), "c".repeat(w), "c".repeat(w)]
    }

    pub fn ice_cream(left: f64) -> Rows {
        let scoop: [&str; 3] = if left > 0.66 {
            ["..PP.", ".PPPP", "PPPPP"]
        } else if left > 0.33 {
            [".....", ".PPP.", "PPPPP"]
        } else if left > 0.05 {
            [".....", ".....", ".PPP."]
        } else {
            [".....", ".....", "....."]
        };
        rows(&[scoop[0], scoop[1], scoop[2], "nyyyn", ".nyn.", "..n.."])
    }

    pub const FOOD: [&[&str]; 3] = [
        &["..G.", ".rr.", "rrrr", "rrrr", ".rr."], // apple
        &[".yy.", "ynyy", "yyny", ".yy."],         // cookie
        &["..i..", ".iii.", "iiiii", "ikkki"],     // rice ball
    ];
    pub const FLOWER: &[&str] = &["..P..", ".PyP.", "..P..", "..G..", ".GG..", "..G.."];
    pub const CLOUD: &[&str] = &["...kkkk...", "..kWWWWk..", ".kWWWWWWkk", "kWWWWWWWWk", ".kkkkkkkk."];
    pub const SOIL: &[&str] = &["....nnnnnnnnnnnn....", "..nnnNnnnnNnnnnNnn..", "nnnnnnnnnnnnnnnnnnnn"];
    pub const BUTTERFLY_A: &[&str] = &["P...P", "PPkPP", ".PkP."];
    pub const BUTTERFLY_B: &[&str] = &["..k..", "PPkPP", "P.k.P"];
    pub const BALL: &[&str] = &["rr", "rr"];

    // Typing activities
    pub const PILE: &[&str] = &["..nnnn..", ".nnNnnn.", "nnnnnNnn"];
    pub const SHOVEL: &[&str] = &["n....", ".n...", "..n..", "..ggg", "..gg."];
    /// Red-framed glasses on a half-size grid (2 pt cells), so each thin rim fits around an eye
    /// (eyes at sprite columns 9 and 12). Starts at half-column 10; the arm runs back toward the ear.
    pub const GLASSES: &[&str] = &[".......rrrr..rrrr", "rrrrrrrr..rrrr..r", ".......r..r..r..r",
                                   ".......r..r..r..r", ".......r..r..r..r", ".......rrrr..rrrr"];

    // Mochi
    pub const YARN_A: &[&str] = &[".PP.", "PkPP", "PPkP", ".PP."];
    pub const YARN_B: &[&str] = &[".PP.", "PPkP", "PkPP", ".PP."];
    pub const BOX: &[&str] = &["Uk..............kU", "kkkkkkkkkkkkkkkkkk", "kTTTTTTTUUTTTTTTTk", "kTTTTTTTUUTTTTTTTk",
                               "kTTTTTTTUUTTTTTTTk", "kTTTTTTTTTTTTTTTTk", "kkkkkkkkkkkkkkkkkk"];
    pub const LASER_DOT: &[&str] = &["r"];
    pub const ROD: &[&str] = &["......nn", ".....n.k", "....n..k", "...n...k", "..n....k", ".n.....k", "n......k", ".......r"];
    pub const FISH: &[&str] = &[".ooo.o", "oooooo", ".ooo.o"];

    // Hero pets (Grumbo, Wisp-a-ling, Hexley, Mossback, Sir Clank, Lunette)
    pub const HERO_AXE: &[&str] = &["...kkk", "..kggk", "nnkggk", "..nkk.", "..n...", "..n..."];
    pub const HERO_HELMET: &[&str] = &["...kkkkkkk...", "..kgggggggk..", ".kgggigggggk.", "kgggggggggggk", "kkkkkkkkkkkkk"];
    pub const HERO_STAR: &[&str] = &["..y..", ".yyy.", "yyyyy", ".yyy.", "..y.."];
    pub const HERO_LOG: &[&str] = &[".kkkkkk.", "knnnnNnk", "knNnnnnk", ".kkkkkk."];
    pub const HERO_THREAD_SPOOL: &[&str] = &["kkkkk", "kCCCk", "kccck", "kCCCk", "kkkkk"];
    pub const HERO_KNOT: &[&str] = &[".CC.CC.", "C..C..C", ".CCCCC.", "...C...", "..C.C.."];
    pub const HERO_MOTH: &[&str] = &["i...i", "iikii", ".iki."];
    pub const HERO_WAND: &[&str] = &["..y", "..n", ".n.", "n.."];
    pub const HERO_FROG: &[&str] = &["..kk..kk..", ".kGGkkGGk.", ".kGkGGkGk.", "kGGGGGGGGk", "kGGGrrGGGk", ".kkkkkkkk.",
                                     "kGGk..kGGk"];
    pub const HERO_SHEEP: &[&str] = &["..kk.kk...", ".kWWkWWkk.", "kWWWWWWWwk", "kWWWWWWkwk", ".kWWWWWkk.", "..kkkkkk..",
                                      "..nk..nk.."];
    pub const HERO_PUFF: &[&str] = &["..WW.W..", ".WWWWWW.", "WWWWWWWW", ".WWWWWW.", "..W.WW.."];
    pub const HERO_BOOK: &[&str] = &[".kk.kk.", "kiikiik", "kiisiik", "kiikiik", "krrrrrk"];
    pub const HERO_ROCK: &[&str] = &[".kkk.", "kgggk", "kgsgk", ".kkk."];
    pub const HERO_FLOWER: &[&str] = &["..P..", ".PyP.", "..P..", "..G..", ".GG.."];
    pub const HERO_RAIN: &[&str] = &["c...c", "..c..", "c...c"];
    pub const HERO_Z: &[&str] = &["kkkk", "..k.", ".k..", "kkkk"];
    pub const HERO_KEY_A: &[&str] = &[".kk.kk.", "kyykyyk", ".kkykk.", "...y...", "...y..."];
    pub const HERO_KEY_B: &[&str] = &["...k...", "..kyk..", "...k...", "...y...", "...y..."];
    pub const HERO_SPRING: &[&str] = &[".gggg.", "g.....", ".gggg.", ".....g", ".gggg."];
    pub const HERO_BELL: &[&str] = &["..kk..", ".kyyk.", "kyyyyk", "kkkkkk", "..y..."];
    pub const HERO_PAD: &[&str] = &["kkkkkkkk", "kiiiiiik", "kikikikk", "kiiiiiik", "kkkkkkkk"];
    pub const HERO_STAMP: &[&str] = &[".kkk.", ".knk.", "kkkkk", "kgggk"];
    pub const HERO_PEBBLES: &[&str] = &[".g..g.g.", "gg.ggggg"];
    pub const HERO_SPARK: &[&str] = &[".C.", "CCC", ".C."];
    pub const HERO_PAW: &[&str] = &["kwk", "kwk"];
    pub const LUNETTE_MORTAR: &[&str] = &["kkkkkkkk", "kssiiiik", ".kssssk.", ".kssssk.", "..kkkk.."];
    pub const LUNETTE_MALLET: &[&str] = &["...kkkkk", "...kTTTk", "...kkkkk", ".....n..", ".....n..", "..kwwn..", "..kkkn.."];
    pub const LUNETTE_CAKE: &[&str] = &[".kkk.", "kiWik", "kiWik", ".kkk.", "..w.."];
    pub const LUNETTE_COMET: &[&str] = &["y.........", ".yy.......", "..yyy.....", "....yy....", ".....y...."];
    pub const LUNETTE_CUSHION: &[&str] = &[
        ".......kkkkkkk........", ".....kkyyyyyk.........", "....kyyyyyk...........",
        "...kyyyyyk............", "..kyyyyyk.............", ".kyyyyyk..............",
        ".kyyyyTk..............", "kyyyyyTk..............", "kyyyyyTk..............",
        "kyyyyyTk..............", "kyyyyyyTk.............", "kyyyyyyyTk............",
        ".kyyyyyyyTk...........", ".kyyyyyyyyTkk.........", "..kyyyyyyyyyTkkkkk....",
        "...kyyyyyyyyyyyyyykk..", "....kkyyyyyyyyyyyyyyk.", "......kkkkkkkkkkkkkk..",
    ];
    /// A little glowing crescent and a twinkle for Moona.
    pub const MOONA_MOON: &[&str] = &["..BB.", ".Bb..", "Bb...", "Bb...", ".Bb..", "..BB."];
    pub const MOONA_TWINKLE: &[&str] = &[".B.", "BWB", ".B."];
    /// Lunette's ears folded down for her nap; replaces the top six rows of her sitting frame.
    pub const LUNETTE_TUCKED_EARS: [&str; 6] = ["................", "................", "....kkkkkk......", "...kwwwwwwk.....",
                                                "..kwwwwwwwwkk...", "..kwwwwwwwwwwwk."];

    // Kage the ninja: a smoke bomb, shuriken (two turns of the spin), the target he throws at, the
    // bush he hides in, and the scroll and brush he writes with while you type.
    pub const NINJA_BOMB: &[&str] = &["..y.", ".k..", "kddk", "kddk", ".kk."];
    pub const NINJA_SHURIKEN_A: &[&str] = &["..k..", ".kgk.", "kgigk", ".kgk.", "..k.."];
    pub const NINJA_SHURIKEN_B: &[&str] = &["k...k", ".kgk.", "..i..", ".kgk.", "k...k"];
    pub const NINJA_TARGET: &[&str] = &[
        ".kkkkk.", "kiiiiik", "kirrrik", "kiriiik", "kirrrik", "kiiiiik", ".kkkkk.", "...n...", "...n...", "..nnn..",
    ];
    pub const NINJA_BUSH: &[&str] = &[
        "...LLL..LL..", "..LLGLLLLGL.", ".LLLLLGLLLLL", "LLGLLLLLLGLL", "LLLLLGLLLLLL", "LGLLLLLLGLLL", "LLLLGLLLLLGL",
        ".LLLLLLGLLL.", "..GGGGGGGG..",
    ];
    pub const NINJA_SCROLL: &[&str] = &["n......n", "kiiiiiik", "kikkikik", "kiiiiiik", "n......n"];
    pub const NINJA_BRUSH: &[&str] = &[".n", ".n", ".n", "kk"];

    // Ronin the samurai: his katana, the polishing cloth, a leaf and its two halves, the flash of a
    // cut, a bamboo stalk in two parts, and a cherry petal.
    pub const SAMURAI_KATANA: &[&str] = &["...k........", "rrrkiiiiiiii", "...k........"];
    pub const SAMURAI_CLOTH: &[&str] = &["WW", "WW"];
    pub const SAMURAI_LEAF: &[&str] = &[".LL.", "LGLL", ".LL."];
    pub const SAMURAI_LEAF_L: &[&str] = &[".L", "LG", ".L"];
    pub const SAMURAI_LEAF_R: &[&str] = &["L.", "LL", "L."];
    pub const SAMURAI_SLASH: &[&str] = &["WWWWWWWWWWWWWW"];
    pub const SAMURAI_BAMBOO_TOP: &[&str] = &[".L.", "LGL", "LGL", "LGL", "GGG", "LGL", "LGL", "LGL"];
    pub const SAMURAI_BAMBOO_BOTTOM: &[&str] = &["LGL", "LGL", "GGG", "LGL", "LGL", "LGL", "GGG", "LGL", "LGL"];
    pub const SAMURAI_PETAL: &[&str] = &["PP", ".P"];

    // Kage's second set: a leaf that circles him while he meditates.
    pub const NINJA_LEAF: &[&str] = &[".L", "LG"];
    // Ronin's second set: a tea bowl and its steam, the ring of a whirlwind slash, a fly (wings up and
    // down) and the chopsticks he catches it with.
    pub const SAMURAI_TEA: &[&str] = &["kGGGk", ".kkk."];
    pub const SAMURAI_STEAM_A: &[&str] = &["W.", ".W", "W."];
    pub const SAMURAI_STEAM_B: &[&str] = &[".W", "W.", ".W"];
    pub const SAMURAI_RING: &[&str] = &[
        "......WWWWWWWW......",
        "...WWW........WWW...",
        ".WW..............WW.",
        "W..................W",
        ".WW..............WW.",
        "...WWW........WWW...",
        "......WWWWWWWW......",
    ];
    pub const SAMURAI_FLY_A: &[&str] = &["i.i", ".k."];
    pub const SAMURAI_FLY_B: &[&str] = &["...", "iki"];
    pub const SAMURAI_CHOPSTICKS: &[&str] = &["...nn", "..nn.", ".nn..", "nn..."];

    // Pastimes (what pets other than Mochi and Puddle do instead of reading): a yo-yo, knitting
    // needles (two clicks), a butterfly (wings open and shut), juggling balls, a bubble wand and a bubble.
    pub const PASTIME_YOYO: &[&str] = &[".r.", "rkr", ".r."];
    pub const PASTIME_NEEDLES_A: &[&str] = &["n...n", ".n.n.", "..n.."];
    pub const PASTIME_NEEDLES_B: &[&str] = &[".n.n.", ".n.n.", "..n.."];
    pub const PASTIME_BUTTERFLY_A: &[&str] = &["P.P", ".k.", "P.P"];
    pub const PASTIME_BUTTERFLY_B: &[&str] = &["...", "PkP", "..."];
    pub const PASTIME_BALL_R: &[&str] = &["rr", "rr"];
    pub const PASTIME_BALL_Y: &[&str] = &["yy", "yy"];
    pub const PASTIME_BALL_C: &[&str] = &["cc", "cc"];
    pub const PASTIME_WAND: &[&str] = &[".ii.", "i..i", ".ii.", ".n..", "n..."];
    pub const PASTIME_BUBBLE: &[&str] = &[".c.", "c.c", ".c."];
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
    pub const DROP: &[&str] = &["...k...", "..kck..", ".kccck.", "kcCccck", "kcCccck", "kccccck", ".kkkkk."];
    pub const RICE: &[&str] = &["..kkkkk..", ".kiiiiik.", "kiiiiiiik", "kkkkkkkkk", "koooooook", ".koooook.", "..kkkkk.."];
    pub const STEW: &[&str] = &["..g...g..", "...g...g.", "..g...g..", "kkkkkkkkk", "dkoyoNokd", ".kdddddk.", ".kdddddk.", "..kkkkk.."];
    pub const STRETCH: &[&str] = &["k.....k", ".k.o.k.", "..kok..", "...k...", "...k...", "..k.k..", ".k...k."];
    pub const TULIP: &[&str] = super::trick_art::FLOWER;
    pub const HEART: &[&str] = &[".PP.PP.", "PPPPPPP", "PPPPPPP", ".PPPPP.", "..PPP..", "...P..."];
}

/// Shifts the hue of the given palette keys (HSB, like NSColor), wrapping around.
pub fn hue_shifted(palette: &Palette, amount: f64, keys: &[char]) -> Palette {
    let mut out = *palette;
    for &key in keys {
        let Some(c) = palette.get(key as u8) else { continue };
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
        out.set(key, Some(Color::new((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8, c.a)));
    }
    out
}
