//! Grumbo the warrior.

mod art;
mod tricks;
mod typing;

pub(crate) use art::*;
pub(crate) use petkit::prelude::*;

petkit::export_pet!(pet);

/// The whole pet: what it looks like, its tricks, and what it does while you type.
pub fn pet() -> PetDef {
    PetDef::new(info()).trick(tricks::spin::Spin).trick(tricks::helmet::Helmet).trick(tricks::chop::Chop).typing(typing::Stamping)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[
            ('k', hex(0x33293d)),
            ('w', hex(0xe9bd8e)),
            ('s', hex(0xba886c)),
            ('h', hex(0x96909e)),
            ('H', hex(0xc4bfca)),
            ('v', hex(0x827565)),
        ]),
        body: [
            "................",
            "....kkkkkkkk....",
            "...khhhhhhhhkk..",
            "..khhHhhhhhhhhk.",
            "..khhhhhhhhhhhk.",
            ".kkkkkkkkkkkkkk.",
            ".kswwwwwwwwwwwk.",
            ".kswwwwwwwwwwwk.",
            ".kswwwwwwswwwsk.",
            ".kvvwwwwwwwwvvk.",
            ".kvvvvvvvvvvvvk.",
            "..kvvvvHHvvvvk..",
            "...kkkkkkkkkk...",
        ],
        feet: ["...kvk....kvk...", ".....kvkkvk....."],
        pastime: Some(Pastime::Juggle),
        ..PetInfo::new("grumbo", "Grumbo the warrior", Category::Hero)
    }
}
