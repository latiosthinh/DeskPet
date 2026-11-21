//! Lunette the moon rabbit.

mod art;
mod tricks;
mod typing;

pub(crate) use art::*;
pub(crate) use petkit::prelude::*;

petkit::export_pet!(pet);

/// The whole pet: what it looks like, its tricks, and what it does while you type.
pub fn pet() -> PetDef {
    PetDef::new(info()).trick(tricks::rice::Rice).trick(tricks::wish::Wish).trick(tricks::nap::Nap).typing(typing::Mortar)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[('k', hex(0x33293d)), ('w', hex(0xfff3df)), ('s', hex(0xd9c9e8)), ('p', hex(0xe8a5b7))]),
        body: [
            "....kk...kk.....",
            "...kwpk.kpwk....",
            "...kwpk.kpwk....",
            "...kwpk.kpwk....",
            "...kwwkkkwwkk...",
            "..kwwwwwwwwwwwk.",
            ".kwwwwwwwwwwwwk.",
            ".kwwwwwwwwwwwwk.",
            "kwwkwwwwwwwwwpk.",
            "kwwkswwwwwwwwwk.",
            ".kkkswwwwwwwwwk.",
            "...ksssswwwwwk..",
            "....kkkkkkkkk...",
        ],
        feet: ["....kwk....kwk..", "......kwkkwk...."],
        pastime: Some(Pastime::Knit),
        ..PetInfo::new("lunette", "Lunette the moon rabbit", Category::Hero)
    }
}
