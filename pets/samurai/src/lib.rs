//! Ronin the samurai.

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
        .trick(tricks::draw::Draw)
        .trick(tricks::bow::Bow)
        .trick(tricks::bamboo::Bamboo)
        .trick(tricks::tea::Tea)
        .trick(tricks::spin::Spin)
        .trick(tricks::fly::Fly)
        .typing(typing::Polish)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[
            ('k', hex(0x2a2230)),
            ('w', hex(0xc0392b)),
            ('s', hex(0xf0c8a0)),
            ('h', hex(0x34343f)),
            ('H', hex(0x5a5a6a)),
            ('R', hex(0xe8b84a)),
            ('y', hex(0xf5d060)),
        ]),
        body: [
            "...y.......y....",
            "....y.....y.....",
            ".....yyyyy......",
            "....kkkkkkkk....",
            "...khhhhhhhhk...",
            "..khhhhhhhhhhk..",
            ".khhkssssssssk..",
            ".khhkssssssssk..",
            ".kHHkssssssssk..",
            ".kwwwkkkkkkkwk..",
            ".kwwwwRRwwwwwk..",
            ".kwRwwwwwwRwwk..",
            "..kkkkkkkkkkkk..",
        ],
        feet: ["...khk....khk...", ".....khkkhk....."],
        paw: b's',
        pastime: Some(Pastime::Flower),
        ..PetInfo::new("samurai", "Ronin the samurai", Category::Hero)
    }
}
