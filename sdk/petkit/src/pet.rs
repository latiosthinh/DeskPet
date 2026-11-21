//! Describing a pet: what it looks like ([`PetInfo`]), its tricks ([`Trick`]) and what it does while
//! you type ([`Typing`]), put together in a [`PetDef`].

use crate::art::{Eyes, Frame, Palette, Picture, Pose, PET_SIZE};
use crate::look::Look;
use crate::stage::Stage;

/// Which shelf a pet sits on, in the app and on the website.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    /// Small everyday animals and creatures (Mochi, Ember, Pip, …).
    Critter,
    /// Little adventurers with gear (Grumbo, Kage, Ronin, …).
    Hero,
    /// Big detailed pets drawn from pictures (Moona, Orbis).
    Legend,
}

impl Category {
    pub fn id(self) -> &'static str {
        match self {
            Category::Critter => "critter",
            Category::Hero => "hero",
            Category::Legend => "legend",
        }
    }
}

/// What a pet does now and then to pass the time, sitting down with it for a while.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pastime {
    Book,    // reads
    Yoyo,    // plays with a yo-yo
    Knit,    // knits a scarf
    Flower,  // sits by a flower while a butterfly flutters round it
    Juggle,  // juggles three balls
    Bubbles, // blows bubbles
}

/// Seconds between the tricks a pet does on its own (4–8 minutes), unless it says otherwise.
pub const TRICK_GAP: (f64, f64) = (4.0 * 60.0, 8.0 * 60.0);

/// An empty 16-column row, for bodies that leave rows blank.
pub const EMPTY_ROW: &str = "................";

/// Everything about a pet that isn't code: its name, colours, body and habits.
///
/// Start from [`PetInfo::new`] and set what your pet needs:
///
/// ```ignore
/// PetInfo {
///     palette: Palette::new(&[('k', hex(0x221d2e)), ('w', hex(0x3d4870))]),
///     body: [ /* 13 rows of 16 characters */ ],
///     feet: ["....kwk..kwk....", "......kwkkwk...."],
///     pastime: Some(Pastime::Yoyo),
///     ..PetInfo::new("ninja", "Kage the ninja", Category::Hero)
/// }
/// ```
pub struct PetInfo {
    /// Short lowercase id, unique across all pets: `[a-z0-9-]`, also the file name (`<id>.deskpet`).
    pub id: &'static str,
    /// Shown in menus: "Kage the ninja". The part before " the " is used on its own ("Kage").
    pub name: &'static str,
    pub category: Category,
    /// One-line description for the website.
    pub description: &'static str,
    /// Who made it, for the website's credit line.
    pub author: &'static str,
    /// The module's version; the app updates an installed pet when the website has a newer one.
    pub version: u32,
    /// Colours for the body's characters. 'k' (outline) is also used for props.
    pub palette: Palette,
    /// The 16×16 body without eyes and feet: 13 rows of 16 characters, '.' is see-through.
    /// Column 14 is the front outline, where props are held.
    pub body: [&'static str; 13],
    /// Two rows of feet, swapped while walking.
    pub feet: [&'static str; 2],
    /// A pet without legs keeps this row under it when it sits (otherwise sitting drops it a row).
    pub sit_base: Option<&'static str>,
    /// Which body colour paws are drawn in on props (the laptop).
    pub paw: u8,
    /// Columns of the two eyes, drawn on rows 6–7.
    pub eye_cols: [usize; 2],
    /// Set instead of `body` for a pet drawn from detailed pictures (see [`Picture`]).
    pub picture: Option<Picture>,
    /// Sweeps with a broom now and then.
    pub sweeps: bool,
    /// Bobs up and down while walking instead of stepping.
    pub floats: bool,
    /// Drinks when reminded about water (otherwise just waits with you).
    pub drinks: bool,
    pub pastime: Option<Pastime>,
    /// Seconds between the tricks it does on its own.
    pub trick_gap: (f64, f64),
    /// Trick ids: picked for about half its own tricks / done each time it turns round to walk the
    /// other way / done each time it wakes up / done on a click instead of being petted / done on a
    /// right-click instead of opening the menu.
    pub favorite_trick: Option<&'static str>,
    pub turn_trick: Option<&'static str>,
    pub wake_trick: Option<&'static str>,
    pub click_trick: Option<&'static str>,
    pub right_click_trick: Option<&'static str>,
}

impl PetInfo {
    /// A pet with everything else at its default: no body yet, eyes at columns 9 and 12, paws in 'w'.
    pub fn new(id: &'static str, name: &'static str, category: Category) -> PetInfo {
        PetInfo {
            id,
            name,
            category,
            description: "",
            author: "DeskPet",
            version: 1,
            palette: Palette::new(&[]),
            body: [EMPTY_ROW; 13],
            feet: [EMPTY_ROW; 2],
            sit_base: None,
            paw: b'w',
            eye_cols: [9, 12],
            picture: None,
            sweeps: false,
            floats: false,
            drinks: true,
            pastime: None,
            trick_gap: TRICK_GAP,
            favorite_trick: None,
            turn_trick: None,
            wake_trick: None,
            click_trick: None,
            right_click_trick: None,
        }
    }

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

    /// The body with eyes drawn in.
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

    /// The whole pet in one pose. For a picture pet this is a marker frame that names the picture.
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

/// One trick. Make a struct for it (with fields for anything it needs to remember while it plays)
/// and implement this. `t` is seconds since the trick started.
///
/// - [`step`](Trick::step) runs every frame: say things, move the pet, throw particles.
/// - [`look`](Trick::look) says what to draw right now. Keep it free of side effects: it may be called
///   more than once a frame, or not at all.
pub trait Trick {
    /// Short id, unique within the pet: "smoke".
    fn id(&self) -> &'static str;
    /// Shown in the Tricks menu: "Smoke Bomb".
    fn title(&self) -> &'static str;
    /// Seconds the trick lasts.
    fn duration(&self) -> f64 {
        8.0
    }
    /// Called when the trick starts, to set up its own fields.
    fn start(&mut self, _cx: &mut Stage) {}
    fn step(&mut self, _cx: &mut Stage, _t: f64, _dt: f64) {}
    fn look(&self, cx: &Stage, _t: f64) -> Look {
        cx.look()
    }
}

/// What a pet does while you type. The default is a laptop.
pub trait Typing {
    /// Called each time typing starts.
    fn begin(&mut self, _cx: &mut Stage) {}
    fn step(&mut self, _cx: &mut Stage, _t: f64, _dt: f64) {}
    fn look(&self, cx: &Stage, t: f64) -> Look {
        crate::engine::laptop_look(cx, t)
    }
}

/// Types on a little laptop.
pub struct Laptop;
impl Typing for Laptop {}

/// A whole pet: its description, its tricks, what it does while you type, and anything it always
/// draws (Sir Clank's wind-up key).
pub struct PetDef {
    pub info: PetInfo,
    pub tricks: Vec<Box<dyn Trick>>,
    pub typing: Box<dyn Typing>,
    /// Called last for every frame, to add what the pet always has on (drawn after its trick or activity).
    pub decorate: Option<fn(&Stage, &mut Look)>,
}

impl PetDef {
    pub fn new(info: PetInfo) -> PetDef {
        PetDef { info, tricks: vec![], typing: Box::new(Laptop), decorate: None }
    }
    pub fn trick(mut self, t: impl Trick + 'static) -> PetDef {
        self.tricks.push(Box::new(t));
        self
    }
    pub fn typing(mut self, t: impl Typing + 'static) -> PetDef {
        self.typing = Box::new(t);
        self
    }
    pub fn decorate(mut self, f: fn(&Stage, &mut Look)) -> PetDef {
        self.decorate = Some(f);
        self
    }
}
