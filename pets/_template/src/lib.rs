//! __NAME__.
//!
//! Start here: change the body and colours below, then the tricks in `tricks/`. Try it with
//! `windows\try-pet.ps1 __ID__ -Tricks`, check it with `windows\check-pet.ps1 __ID__`.
//! CONTRIBUTING-PETS.md walks through everything.

mod art;
mod tricks;

pub(crate) use art::*;
pub(crate) use petkit::prelude::*;

petkit::export_pet!(pet);

/// The whole pet: what it looks like and its tricks, in menu order. Add `.typing(...)` to give it
/// something of its own to do while you type (otherwise it types on a laptop).
pub fn pet() -> PetDef {
    PetDef::new(info()).trick(tricks::hop::Hop).trick(tricks::flower::Flower::default())
}

fn info() -> PetInfo {
    PetInfo {
        description: "A round little blob who hops for joy and picks you flowers.",
        author: "__AUTHOR__",
        // One character per colour. '.' is see-through; 'k' is the outline (eyes are drawn in it).
        palette: Palette::new(&[
            ('k', hex(0x33293d)), // outline
            ('b', hex(0x8fd3c1)), // body
            ('B', hex(0xc6eee2)), // shine
            ('s', hex(0x62a99a)), // shade
            ('p', hex(0xf29bb0)), // cheeks
        ]),
        // 13 rows of 16, facing right. The eyes go in columns 9 and 12 of rows 6 and 7, so keep
        // those as body colour. Column 14 is the front edge, where the pet holds things.
        body: [
            "................",
            "................",
            "................",
            "................",
            ".....kkkkkk.....",
            "...kkbbbbbbkk...",
            "..kbBBbbbbbbbk..",
            ".kbBbbbbbbbbbbk.",
            ".kbbbbbbbbbbbbk.",
            ".kbbbbbbbpbbpbk.",
            ".ksbbbbbbbbbbbk.",
            "..ksssbbbbbbsk..",
            "...kkkkkkkkkk...",
        ],
        // Two rows of feet, swapped while it walks.
        feet: ["...kbk....kbk...", ".....kbk.kbk...."],
        ..PetInfo::new("__ID__", "__NAME__", Category::Critter)
    }
}
