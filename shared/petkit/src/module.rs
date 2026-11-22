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
    /// The last panic's message, for the app to show when a call fails.
    static PANIC: RefCell<String> = const { RefCell::new(String::new()) };
}

/// `API_VERSION` as one number: major in the high 16 bits.
pub fn api() -> u32 {
    let (major, minor) = crate::API_VERSION;
    major << 16 | minor
}

pub fn new(mut def: PetDef, version: &'static str, now: f64, seed: u64) {
    // A panic stops the module (WebAssembly can't unwind); keep its message so the app and petshot
    // can say what went wrong instead of just "unreachable".
    std::panic::set_hook(Box::new(|info| {
        PANIC.with(|p| {
            if let Ok(mut p) = p.try_borrow_mut() {
                *p = info.to_string();
            }
        })
    }));
    def.info.version = version;
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
    BRAIN.with(|b| f(b.borrow_mut().as_mut().expect("deskpet: deskpet_new wasn't called")))
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

/// The last panic's message (UTF-8, empty if none) as `(address << 32) | length`.
pub fn panic_message() -> u64 {
    PANIC.with(|p| {
        let p = p.borrow();
        (p.as_ptr() as u64) << 32 | p.len() as u64
    })
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
                m::new(super::$make(), env!("CARGO_PKG_VERSION"), now, seed)
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
            #[no_mangle]
            pub extern "C" fn deskpet_panic() -> u64 {
                m::panic_message()
            }
        }
    };
}
