//! Mossback the pebble giant.

mod art;
mod tricks;
mod typing;

pub(crate) use art::*;
pub(crate) use petkit::prelude::*;

petkit::export_pet!(pet);

/// The whole pet: what it looks like, its tricks, and what it does while you type.
pub fn pet() -> PetDef {
    PetDef::new(info()).trick(tricks::toss::Toss).trick(tricks::bloom::Bloom).trick(tricks::rain::Rain).typing(typing::Pebbles)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[
            ('k', hex(0x33293d)),
            ('w', hex(0xc0b7a3)),
            ('s', hex(0x938b7f)),
            ('m', hex(0x82965c)),
            ('M', hex(0xb8c68a)),
            ('b', hex(0x796b57)),
        ]),
        body: [
            "................",
            "......kkk.......",
            "...kkkMmMkkk....",
            "..kmMmmmMmmmk...",
            ".kmmmmmMmmmmmk..",
            ".kwwwwwwwwwwwwk.",
            ".kswwwwwwwwwwwk.",
            ".kswwwwwwwwwwwk.",
            ".kswwwwwswwwwsk.",
            ".kswbwwwwwwwwwk.",
            ".kssbwwwbwwwwwk.",
            "..ksswwwbwwwwk..",
            "...kkkkkkkkkk...",
        ],
        feet: ["..kbk.....kbk...", "....kbk.kbk....."],
        pastime: Some(Pastime::Flower),
        ..PetInfo::new("mossback", "Mossback the pebble giant", Category::Hero)
    }
}
