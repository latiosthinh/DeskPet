//! Wisp-a-ling the spirit.

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
    PetDef::new(info()).trick(tricks::thread::Thread).trick(tricks::knot::Knot).trick(tricks::moth::Moth).typing(typing::Spool)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[('k', hex(0x33293d)), ('w', hex(0xf7edb0)), ('s', hex(0xe9b878)), ('h', hex(0xfff7d8)), ('p', hex(0xe7a98d))]),
        body: [
            ".....kk.........",
            "....kwhk........",
            "...kwwk.........",
            "...kwwkkkkkk....",
            "..kwwwwwwwhhkk..",
            ".kwwwwwwwwwwwwk.",
            ".kwwwwwwwwwwwwk.",
            ".kswwwwwwwwwwwk.",
            "..kswwwwpwwwwpk.",
            "...kswwwwwwwwwk.",
            "..kwswwwwwwwwk..",
            ".kwkkswwwwwwk...",
            "..k..kkkkkkk....",
        ],
        feet: ["......kk..kk....", ".....kk..kk....."],
        sit_base: Some("......kk..kk...."),
        floats: true,
        pastime: Some(Pastime::Juggle),
        description: "A floating little spirit who reaches for your pointer and gets tied up in its own thread.",
        ..PetInfo::new("wispaling", "Wisp-a-ling the spirit", Category::Hero)
    }
}
