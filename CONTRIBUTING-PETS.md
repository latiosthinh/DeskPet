# Making a pet for DeskPet

Every DeskPet pet except the three that ship with the app is a small module, `<id>.deskpet`, that people
download from the pet's page on the DeskPet website. You can make one: a pet is a Rust crate of a few
small files, and you never touch the app or anyone else's pet.

A pet module runs in a sandbox (WebAssembly). It can't read files, use the network or see anything on the
computer: it only gets the time and where the pointer is, and says what to draw.

- [Quick start](#quick-start)
- [What's in a pet](#whats-in-a-pet)
- [Drawing the pet](#drawing-the-pet)
- [Your first trick](#your-first-trick)
- [Reference](#reference): `Stage`, `Look`, props and sprites, typing and pastimes
- [Rules the checker enforces](#rules-the-checker-enforces)
- [Sending it in](#sending-it-in)

## Quick start

You need [Rust](https://rustup.rs) (stable) on Windows, with the Visual Studio C++ build tools. The scripts add
the WebAssembly target (`wasm32-unknown-unknown`) the first time. From the top of the repository:

```powershell
powershell -ExecutionPolicy Bypass -File windows\new-pet.ps1 bean "Bean the blob" -Author "Your Name"
powershell -ExecutionPolicy Bypass -File windows\try-pet.ps1 bean -Tricks
powershell -ExecutionPolicy Bypass -File windows\check-pet.ps1 bean -Gifs
```

1. **`new-pet.ps1`** copies `pets\_template` to `pets\bean`: a little blob with a body, colours and two
   example tricks, ready to change. The id is lowercase letters, digits and `-`. It is the pet's file name
   and its address on the website, and it never changes.
2. **`try-pet.ps1`** builds the pet and puts it on your taskbar without installing it. `-Tricks` plays every
   trick in turn, `-Typing` shows what it does while you type, and `-Pastime` shows its pastime. Run it again
   after each change.
3. **`check-pet.ps1`** plays everything the pet can do off screen and reports anything wrong. For example:
   - an art row that isn't 16 characters;
   - a colour missing from the palette;
   - a trick that crashes, with the file and line;
   - a missing description.

   `-Gifs` also writes the website's GIFs to `build\gifs\bean`, one per trick plus `hero.gif`.

## What's in a pet

```
pets/bean/
├─ Cargo.toml          name "pet-bean", version, licence; depends only on petkit
└─ src/
   ├─ lib.rs           the pet: its info (name, colours, body) and which tricks it has
   ├─ art.rs           its own pixel art for props
   └─ tricks/
      ├─ mod.rs        lists the trick files
      ├─ hop.rs        one file per trick
      └─ flower.rs
```

`lib.rs` ends up looking like this:

```rust
mod art;
mod tricks;

pub(crate) use art::*;
pub(crate) use petkit::prelude::*;   // everything a pet uses

petkit::export_pet!(pet);            // makes this crate a DeskPet module

pub fn pet() -> PetDef {
    PetDef::new(info()).trick(tricks::hop::Hop).trick(tricks::flower::Flower::default())
}

fn info() -> PetInfo {
    PetInfo {
        description: "A round little blob who hops for joy and picks you flowers.",
        author: "Your Name",
        palette: Palette::new(&[('k', hex(0x33293d)), ('b', hex(0x8fd3c1)), /* ... */]),
        body: [/* 13 rows of 16 characters */],
        feet: ["...kbk....kbk...", ".....kbk.kbk...."],
        ..PetInfo::new("bean", "Bean the blob", Category::Critter)
    }
}
```

Everything a pet shares with the others is done for you. That covers walking about, sleeping when you're
away, sitting with you while you type, the water, meal and stretch reminders, being petted, and choosing when
to do a trick (every 4 to 8 minutes). You add what makes your pet itself: how it looks, and its tricks.

**Category** is the shelf it goes on: `Critter` (animals and creatures), `Hero` (little people) or `Legend`
(bigger, fancier pets).

**Version:** bump `version` in the pet's `Cargo.toml` whenever you change it. The app updates installed copies
when the website has a newer version.

## Drawing the pet

A pet is 16×16 art pixels, facing right, drawn with one character per pixel:

- **`body`**: 13 rows of exactly 16 characters. `.` is see-through.
- **`feet`**: two rows of 16 that take turns while it walks. They make rows 14 and 15 of the sprite.
- **`palette`**: the colour of each character, for example `('b', hex(0x8fd3c1))`. It must have `k`, the
  outline colour, because the eyes are drawn in it.
- **Eyes** go in columns 9 and 12 of rows 6 and 7 (`eye_cols` changes the columns). Leave those cells as body
  colour: the app draws the eyes open, closed or happy.
- **Column 14** is the front edge. Props the pet holds sit there.

Other `PetInfo` settings, all optional:

| Field | What it does |
|---|---|
| `floats: true` | Bobs along instead of stepping (ghosts, slimes). |
| `sit_base: Some("…")` | For a pet without legs: the row kept under it when it sits. Otherwise sitting drops it one row. |
| `paw: b'w'` | Which body colour its paws use on the laptop. |
| `sweeps: true` | Sweeps with a broom now and then. |
| `drinks: true` | Drinks when reminded about water; otherwise it waits with you. |
| `pastime: Some(Pastime::Yoyo)` | What it does when it sits for a while: see [Typing and pastimes](#typing-and-pastimes). |
| `favorite_trick: Some("hop")` | Picked for about half of the tricks it does on its own. |
| `click_trick`, `right_click_trick` | A trick done on a click, instead of being petted, or on a right-click, instead of opening the menu. |
| `wake_trick`, `turn_trick` | A trick done each time it wakes up, or each time it turns round. |
| `trick_gap: (min, max)` | Seconds between tricks it does on its own. |

A pet drawn from bigger, detailed pictures instead of the 16×16 grid (like Moona) sets `picture`. Look at
`pets/moona` if you want to do that.

## Your first trick

A trick is a struct that implements `Trick`. Here is the template's whole `hop.rs`:

```rust
use crate::*;

pub struct Hop;

impl Trick for Hop {
    fn id(&self) -> &'static str { "hop" }            // unique within the pet; names its GIF
    fn title(&self) -> &'static str { "Hop for Joy" } // shown in the Tricks menu
    fn duration(&self) -> f64 { 4.0 }                 // seconds (8 if you leave it out)

    // Things that happen. Called every frame; `t` is seconds since the trick started.
    fn step(&mut self, cx: &mut Stage, t: f64, _dt: f64) {
        if cx.once(1) {
            cx.say("yay!", 1.5);
        }
        if t > 0.5 && cx.every(2, 0.5) {
            cx.spawn_above_head("✦", sys_color::YELLOW, 1, 1.0);
        }
    }

    // How it looks at time `t`. Start from `cx.look()` (sitting, blinking) and change it.
    fn look(&self, cx: &Stage, t: f64) -> Look {
        let mut look = cx.look();
        look.frame = cx.pet.frame(Eyes::Happy, Pose::Stand(0));
        look.y_offset = ((t * 2.0 * PI).sin().abs() * 4.0).round();
        look
    }
}
```

Then:

1. Add it to `tricks/mod.rs` (`pub mod hop;`) and to `pet()` (`.trick(tricks::hop::Hop)`).
2. Run `try-pet.ps1 bean -Tricks` to see it.

**`step` and `look`.** `step` *does* things: it speaks, moves the pet, and starts particles. It may change the
trick's own fields. `look` only *describes* the picture for time `t`. Keep anything random or one-off in
`step`, guarded by `once` or `every` so it happens when you mean it to.

**A trick that remembers something** gives its struct fields, and sets them up in `start`, which runs each
time the trick begins. The same value is reused every time the trick plays. See `tricks/flower.rs` in the
template, which walks to the middle of the pen and then holds out a flower.

## Reference

### Stage (`cx`)

`cx` is the pet's world. Useful fields: `cx.pet` (its `PetInfo`), `cx.pet_x` (left edge of the sprite, in
points), `cx.facing_left`, `cx.width` and `cx.height` (the pen), `cx.mouse` (the pointer, in pen
coordinates), `cx.now`.

| Verb | What it does |
|---|---|
| `cx.once(id)` | True the first time it's asked for each number `id` during a trick. |
| `cx.every(id, seconds)` | True once every `seconds` for each timer number `id`. |
| `cx.say(text, seconds)` | A speech bubble. `cx.say_with(text, seconds, Some(icon))` adds a little picture. |
| `cx.spawn(text, color, at, size)` | A particle (a symbol like `"♥"`, `"✦"`, `"♪"`) that floats up and fades. |
| `cx.spawn_above_head(text, color, count, life)` | Particles from above the pet's head. |
| `cx.spawn_full(text, color, at, size, vx, vy, life, ay)` | A particle with its own speed ranges and gravity. |
| `cx.move_toward(x, speed, dt)` | Walks toward `x` at `speed` points a second, facing it; true on arrival. |
| `cx.random_target(x, reach)` | A random spot in the pen within `reach` of `x`. |
| `cx.min_pet_x()`, `cx.max_pet_x()` | How far the pet can go either way. |
| `cx.pet_center_x()`, `cx.center_x(col)`, `cx.x_for_column(col)` | Positions in points, honouring which way it faces. |
| `cx.alt(period)`, `cx.step_of(period)` | Flip between two states every `period` seconds (for two-frame animations). |
| `cx.blink_eyes()` | Open eyes that blink now and then. |
| `cx.look()` | The pet sitting and blinking, as a `Look` to start from. |
| `cx.pet.frame(eyes, pose)` | The pet's 16×16 frame: `Eyes::Open`, `Closed` or `Happy`; `Pose::Stand(0 or 1)` or `Pose::Sit`. |
| `cx.placed(grid, dx, up)` | A prop-coloured sprite placed in front of the pet. |
| `cx.frame_copy(&frame, dx, up)` | A copy of the pet drawn somewhere else (clones, reflections). |
| `cx.restart_trick()` | Starts this trick again from the beginning, after this frame. |

Random numbers: `rand(lo, hi)`, `rand01()` and `rand_index(n)`. There's no clock or system time in a pet; use
`t` and `cx.now`.

### Look

| Field | What it is |
|---|---|
| `frame` | The 16×16 frame to draw (from `cx.pet.frame(...)`, or your own changes to one). |
| `palette` | Its colours: the pet's own unless you change them. `palette.faded(0.5)` makes it half see-through. |
| `y_offset`, `x_offset` | Art pixels up (negative sinks into the ground) and forward, for hops and recoils. |
| `scale` | Grows or shrinks the pet around its middle. |
| `props` | Things it holds: `prop(rows(GRID), column, lift)`. They turn with the pet. |
| `free`, `back` | Sprites anywhere in the pen, in front of or behind it: `free(rows(GRID), x, y, PIXEL)`. |
| `twin` | A second pet beside it (Puddle splitting in two). |
| `beam` | A torch beam from a point at an angle. |

### Props and sprites

Props are drawn in the shared prop colours (`prop_palette()` in `sdk/petkit/src/art.rs`):

| Character | Colour |
|---|---|
| `k` | outline |
| `w` | the pet's paw |
| `i` | white |
| `y` | yellow |
| `r` | red |
| `o` | orange |
| `P` | pink |
| `G` | green stem |
| `L` | leaf green |
| `c`, `C` | water |
| `b`, `B` | light blue |
| `n` | wood |
| `N` | dark soil |
| `g` | grey |
| `s` | pale grey (a page) |
| `d` | dark grey |
| `T`, `U` | cardboard |
| `W` | cloud white |

Put a pet's own grids in its `art.rs`. petkit's `kit` module has some ready-made ones: `HERO_PUFF`,
`HERO_SPARK`, `HERO_STAR`, `HERO_FLOWER`, `TWINKLE` and more. A sprite with `own_colors: true` is drawn in the
pet's palette instead.

### Typing and pastimes

While you type, a pet types on a little laptop unless you give it something of its own:

```rust
pub struct Scribble;
impl Typing for Scribble {
    fn look(&self, cx: &Stage, t: f64) -> Look { /* like a trick's look */ }
}
// in pet():  .typing(typing::Scribble)
```

A pastime is what it does when it sits for a while: `Pastime::Yoyo`, `Knit`, `Flower`, `Juggle` or `Bubbles`.
Reading (`Book`) is kept for Mochi and Puddle.

## Rules the checker enforces

`check-pet.ps1` (petshot) runs these checks, and the website only takes pets that pass:

- **Ids.** The pet id and trick ids use lowercase letters, digits and `-`. Trick ids and titles are unique
  within the pet.
- **Info.** The name is 1 to 40 characters. It has a description and an author. The version looks like
  `1.2.0`.
- **Tricks.** It has at least one trick, and each lasts more than 0 and at most 15 seconds.
- **Art.** Every art row is 16 characters, and every character has a colour.
- **Nothing breaks.** Nothing panics while walking, in any trick, typing, its pastime, a reminder, being petted
  or a right-click.
- **Speed.** No call runs too long. Each frame has a generous instruction budget, so only a loop that never
  ends trips it.
- **Size.** The module is at most 2 MB. A pet is usually 120–200 KB.

If a pet breaks on someone's taskbar, the app swaps in Mochi and says so. Nothing else is affected.

## Sending it in

1. Check it: `check-pet.ps1 bean`.
2. Open a pull request that adds `pets/bean` (and the `Cargo.lock` change). CI builds every pet and runs the
   checks.
3. A maintainer publishes it. They regenerate the website's pet list, GIFs and download files with
   `petshot site build\pets site` and deploy the site (see `site/README.md`).

Pets in this repository are released under the MIT licence: the `license` field in your pet's `Cargo.toml`.
By sending a pet you agree to that, and confirm the art is your own or that you may use it.
