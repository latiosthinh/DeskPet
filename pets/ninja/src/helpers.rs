//! Timings shared by Kage's tricks, in seconds from the start of the trick.

/// Smoke Bomb: when the smoke hides him and when he reappears.
pub const SMOKE_GONE: f64 = 1.5;
pub const SMOKE_BACK: f64 = 4.5;
/// Shuriken: when each one leaves his hand.
pub const SHURIKEN_THROWS: [f64; 3] = [1.0, 2.0, 3.0];
/// Hide: when he pops back out.
pub const HIDE_POP: f64 = 6.0;
/// Clone Trick: when the copies appear and go.
pub const CLONE_IN: f64 = 1.0;
pub const CLONE_OUT: f64 = 5.5;
/// Backflips: when each one leaves the ground, and how long it takes.
pub const FLIPS: [f64; 2] = [1.0, 3.2];
pub const FLIP_TIME: f64 = 0.9;
