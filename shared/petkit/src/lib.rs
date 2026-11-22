//! # petkit: make a pet for DeskPet
//!
//! A pet is a small crate that depends only on `petkit`. It has:
//!
//! - a [`PetInfo`](pet::PetInfo): its name, colours and 16×16 body, drawn as rows of characters;
//! - its tricks, one struct per trick implementing [`Trick`](pet::Trick), each in its own file;
//! - optionally what it does while you type ([`Typing`](pet::Typing)), otherwise a laptop.
//!
//! `pub fn pet() -> PetDef` puts them together. Everything else (walking about, sleeping, reminders,
//! being petted, picking when to do a trick) is shared and lives in [`engine`].
//!
//! Start with `use petkit::prelude::*;`. See `CONTRIBUTING-PETS.md` for a walkthrough.

pub mod art;
pub mod kit;
pub mod look;
pub mod pet;
pub mod random;
pub mod stage;
pub mod wire;

#[doc(hidden)]
pub mod engine;
#[doc(hidden)]
pub mod module;

/// The version of this API. A module records the one it was built with, and the app refuses a module
/// built against a newer major version than it knows.
pub const API_VERSION: (u32, u32) = (1, 0);

/// Goes from `a` to `b` as `t` goes from 0 to 1 (held at the ends).
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Everything a pet uses, in one import.
pub mod prelude {
    pub use crate::art::*;
    pub use crate::kit::*;
    pub use crate::lerp;
    pub use crate::look::*;
    pub use crate::pet::*;
    pub use crate::random::{rand, rand01, rand_index};
    pub use crate::stage::{Mode, Stage};
    pub use std::f64::consts::PI;
}
