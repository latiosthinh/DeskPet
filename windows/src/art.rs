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
}

impl Pet {
    /// "Ember the fox" → "Ember"; "Sir Clank" stays whole.
    pub fn short_name(&self) -> &'static str {
        self.name.split(" the ").next().unwrap_or(self.name)
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
            premium: true,
            wind_key: false,
            floats: false,
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
    ]
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
}

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
            | LunetteNap => 8.0,
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
    /// Lunette's ears folded down for her nap; replaces the top six rows of her sitting frame.
    pub const LUNETTE_TUCKED_EARS: [&str; 6] = ["................", "................", "....kkkkkk......", "...kwwwwwwk.....",
                                                "..kwwwwwwwwkk...", "..kwwwwwwwwwwwk."];
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
