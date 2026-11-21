//! Sir Clank.

mod art;
mod tricks;
mod typing;

pub(crate) use art::*;
pub(crate) use petkit::prelude::*;

petkit::export_pet!(pet);

/// The whole pet: what it looks like, its tricks, and what it does while you type.
pub fn pet() -> PetDef {
    PetDef::new(info()).trick(tricks::wind::Wind).trick(tricks::spring::Spring).trick(tricks::bell::Bell).typing(typing::Checklist).decorate(wind_key)
}

/// His wind-up key sits on his back (it turns in his Wind Up trick, which draws it itself).
fn wind_key(cx: &Stage, look: &mut Look) {
    let winding = cx.mode == Mode::Trick && !cx.dragging && cx.trick_id() == Some("wind");
    if !winding {
        look.back.push(cx.placed(HERO_KEY_A, -4.0 + look.x_offset, 4.0 + look.y_offset));
    }
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[
            ('k', hex(0x33293d)),
            ('w', hex(0xe2dbbf)),
            ('s', hex(0xb5ab96)),
            ('h', hex(0xbc8270)),
            ('H', hex(0xf7efd8)),
            ('y', hex(0xd7b365)),
        ]),
        body: [
            "................",
            ".....kkkkkk.....",
            "....kwHwwwwk....",
            "...kwwwwwwwwk...",
            "..kwwwwwwwwwwk..",
            ".kwwkkkkkkkkwwk.",
            ".kwkwHHHHHHHHwk.",
            ".kwkwHHHHHHHHwk.",
            ".kwwkkkkkkkkwwk.",
            ".kswwwhhhwwwwwk.",
            ".ksswwhHhwwwwwk.",
            "..kssswwwsswwk..",
            "...kkkkkkkkkk...",
        ],
        feet: ["...khk....khk...", ".....khkkhk....."],
        pastime: Some(Pastime::Yoyo),
        description: "A wind-up knight who runs down, needs winding up again and never misses tea time.",
        ..PetInfo::new("sirclank", "Sir Clank", Category::Hero)
    }
}
