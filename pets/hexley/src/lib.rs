//! Hexley the apprentice.

mod art;
mod tricks;
mod typing;

pub(crate) use art::*;
pub(crate) use petkit::prelude::*;

petkit::export_pet!(pet);

/// The whole pet: what it looks like, its tricks, and what it does while you type.
pub fn pet() -> PetDef {
    PetDef::new(info()).trick(tricks::turn_into::FROG).trick(tricks::turn_into::SHEEP).trick(tricks::book::Book).typing(typing::SpellNotes)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[
            ('k', hex(0x33293d)),
            ('w', hex(0xedc6a7)),
            ('s', hex(0xbc947e)),
            ('h', hex(0x729d98)),
            ('H', hex(0xa8c5b5)),
            ('p', hex(0xdba4a7)),
        ]),
        body: [
            "...kk...........",
            "..kHHk..........",
            "...khkkkkkk.....",
            "..khhhhhhhhkk...",
            ".khhHhhhhhhhhhk.",
            ".khhhwwwwwwwwwk.",
            ".khhhwwwwwwwwwk.",
            ".khhhwwwwwwwwwk.",
            ".khhhwwwpwwwwpk.",
            ".khhhhhwwwwhhhk.",
            ".khhhhhHHhhhhhk.",
            "..khhhhHHhhhhk..",
            "...kkkkkkkkkk...",
        ],
        feet: ["...khk....khk...", ".....khkkhk....."],
        pastime: Some(Pastime::Bubbles),
        ..PetInfo::new("hexley", "Hexley the apprentice", Category::Hero)
    }
}
