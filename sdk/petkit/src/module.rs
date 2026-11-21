//! Inside a pet module: keeps the pet's [`Brain`] and answers the app's calls. [`export_pet!`]
//! turns these into the module's exports; pet authors only write `petkit::export_pet!(pet);`.
//!
//! The app writes a request into the inbox (`deskpet_inbox(len)` returns where), then calls a
//! function with its length. Replies are left in the outbox, returned as `(address << 32) | length`.

use crate::engine::{Brain, Command, Input};
use crate::pet::PetDef;
use crate::wire::{decode, encode, Wire};
use std::cell::RefCell;

thread_local! {
    static BRAIN: RefCell<Option<Brain>> = const { RefCell::new(None) };
    static INBOX: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OUTBOX: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// `API_VERSION` as one number: major in the high 16 bits.
pub fn api() -> u32 {
    let (major, minor) = crate::API_VERSION;
    major << 16 | minor
}

pub fn new(def: PetDef, now: f64, seed: u64) {
    crate::random::seed(seed);
    BRAIN.with(|b| *b.borrow_mut() = Some(Brain::new(def, now)));
}

/// Makes room for a request of `len` bytes and says where to write it.
pub fn inbox(len: usize) -> usize {
    INBOX.with(|i| {
        let mut i = i.borrow_mut();
        i.clear();
        i.resize(len, 0);
        i.as_ptr() as usize
    })
}

fn request<T: Wire>(len: usize) -> T {
    let v = INBOX.with(|i| decode(&i.borrow()[..len.min(i.borrow().len())]));
    v.expect("deskpet: request the module can't read")
}

fn reply<T: Wire>(v: &T) -> u64 {
    OUTBOX.with(|o| {
        let mut o = o.borrow_mut();
        *o = encode(v);
        (o.as_ptr() as u64) << 32 | o.len() as u64
    })
}

fn with_brain<R>(f: impl FnOnce(&mut Brain) -> R) -> R {
    BRAIN.with(|b| {
        f(b.borrow_mut()
            .as_mut()
            .expect("deskpet: deskpet_new wasn't called"))
    })
}

pub fn about() -> u64 {
    reply(&with_brain(|b| b.about()))
}

pub fn tick(len: usize) {
    let input: Input = request(len);
    with_brain(|b| b.tick(input));
}

pub fn command(len: usize) -> u32 {
    let c: Command = request(len);
    with_brain(|b| b.command(c)) as u32
}

pub fn render() -> u64 {
    reply(&with_brain(|b| b.render()))
}

/// Makes a pet crate a DeskPet module: `petkit::export_pet!(pet);` with `pub fn pet() -> PetDef`.
/// Only does anything when building for WebAssembly (`--target wasm32-unknown-unknown`).
#[macro_export]
macro_rules! export_pet {
    ($make:ident) => {
        #[cfg(target_arch = "wasm32")]
        mod __deskpet_exports {
            use $crate::module as m;
            #[no_mangle]
            pub extern "C" fn deskpet_api() -> u32 {
                m::api()
            }
            #[no_mangle]
            pub extern "C" fn deskpet_new(now: f64, seed: u64) {
                m::new(super::$make(), now, seed)
            }
            #[no_mangle]
            pub extern "C" fn deskpet_inbox(len: u32) -> u32 {
                m::inbox(len as usize) as u32
            }
            #[no_mangle]
            pub extern "C" fn deskpet_about() -> u64 {
                m::about()
            }
            #[no_mangle]
            pub extern "C" fn deskpet_tick(len: u32) {
                m::tick(len as usize)
            }
            #[no_mangle]
            pub extern "C" fn deskpet_command(len: u32) -> u32 {
                m::command(len as usize)
            }
            #[no_mangle]
            pub extern "C" fn deskpet_render() -> u64 {
                m::render()
            }
        }
    };
}
