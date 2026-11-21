//! Kage the ninja.

mod art;
mod helpers;
mod tricks;
mod typing;

pub(crate) use art::*;
pub(crate) use helpers::*;
pub(crate) use petkit::prelude::*;

petkit::export_pet!(pet);

/// The whole pet: what it looks like, its tricks, and what it does while you type.
pub fn pet() -> PetDef {
    PetDef::new(info())
        .trick(tricks::smoke::Smoke)
        .trick(tricks::shuriken::Shuriken)
        .trick(tricks::hide::Hide)
        .trick(tricks::clones::Clones)
        .trick(tricks::flip::Flip)
        .trick(tricks::meditate::Meditate)
        .typing(typing::Scroll)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[('k', hex(0x221d2e)), ('w', hex(0x3d4870)), ('N', hex(0x5a6894)), ('s', hex(0xf2c9a0)), ('r', hex(0xd8414a))]),
        body: [
            "................",
            ".....kkkkkk.....",
            "....kwwwwwwk....",
            "rr.kwwwwwwwwk...",
            ".rrkrrrrrrrrrk..",
            "..kwwwwwwwwwwwk.",
            "..kwwwwwssssssk.",
            "..kwwwwwssssssk.",
            "..kwwwwwwwwwwwk.",
            "..kwNwwwwwwwwwk.",
            "..kwwNwwwwwwwk..",
            "...kwwwwwwwwk...",
            "....kkkkkkkk....",
        ],
        feet: ["....kwk..kwk....", "......kwkkwk...."],
        pastime: Some(Pastime::Yoyo),
        ..PetInfo::new("ninja", "Kage the ninja", Category::Hero)
    }
}
