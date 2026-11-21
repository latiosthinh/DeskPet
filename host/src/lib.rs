//! The part of DeskPet the Windows and Mac apps share: it runs pet modules (`<id>.deskpet`,
//! WebAssembly) in a sandbox, draws what they render into a pixel buffer, and keeps the list of
//! pets (built in and installed), including installing, removing and updating pet files.
//!
//! Each app only adds what is native to its system: the window, menus, input, settings, text
//! (through [`canvas::TextRaster`]) and HTTP (a function handed to [`catalog::update`]). So any
//! `.deskpet` file runs the same on both.

pub mod canvas;
pub mod catalog;
pub mod module;
pub mod pet;
