//! The bytes that cross between the app and a pet module: a small, fixed binary encoding of the
//! boundary types in [`crate::engine`] (little-endian numbers, length-prefixed strings and lists).
//! Pet authors never use this directly.

use crate::art::{Color, Palette};
use crate::engine::{About, Command, Input, PictureFrame, Reminder, Render};
use crate::look::{FreeSprite, Look, Particle, Prop, Pt, Twin};
use crate::pet::Category;
use std::cell::RefCell;
use std::collections::HashMap;

pub trait Wire: Sized {
    fn put(&self, w: &mut Vec<u8>);
    fn get(r: &mut Reader) -> Option<Self>;
}

/// Reads values back out of a byte slice; every read fails (None) past the end.
pub struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Reader<'a> {
        Reader { bytes, at: 0 }
    }
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(n)?;
        let s = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(s)
    }
    /// A list length, refusing ones longer than the bytes left could hold.
    fn len(&mut self) -> Option<usize> {
        let n = u32::get(self)? as usize;
        (n <= self.bytes.len() - self.at).then_some(n)
    }
}

pub fn encode<T: Wire>(v: &T) -> Vec<u8> {
    let mut w = Vec::new();
    v.put(&mut w);
    w
}

pub fn decode<T: Wire>(bytes: &[u8]) -> Option<T> {
    let mut r = Reader::new(bytes);
    let v = T::get(&mut r)?;
    (r.at == bytes.len()).then_some(v)
}

// MARK: Basics

macro_rules! number {
    ($($t:ty),*) => {$(
        impl Wire for $t {
            fn put(&self, w: &mut Vec<u8>) {
                w.extend_from_slice(&self.to_le_bytes());
            }
            fn get(r: &mut Reader) -> Option<Self> {
                Some(<$t>::from_le_bytes(r.take(std::mem::size_of::<$t>())?.try_into().ok()?))
            }
        }
    )*};
}
number!(u8, u32, u64, f64);

impl Wire for bool {
    fn put(&self, w: &mut Vec<u8>) {
        w.push(*self as u8);
    }
    fn get(r: &mut Reader) -> Option<Self> {
        match u8::get(r)? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}

impl Wire for usize {
    fn put(&self, w: &mut Vec<u8>) {
        (*self as u32).put(w);
    }
    fn get(r: &mut Reader) -> Option<Self> {
        Some(u32::get(r)? as usize)
    }
}

impl Wire for String {
    fn put(&self, w: &mut Vec<u8>) {
        self.len().put(w);
        w.extend_from_slice(self.as_bytes());
    }
    fn get(r: &mut Reader) -> Option<Self> {
        let n = r.len()?;
        String::from_utf8(r.take(n)?.to_vec()).ok()
    }
}

impl<T: Wire> Wire for Vec<T> {
    fn put(&self, w: &mut Vec<u8>) {
        self.len().put(w);
        for v in self {
            v.put(w);
        }
    }
    fn get(r: &mut Reader) -> Option<Self> {
        let n = r.len()?;
        (0..n).map(|_| T::get(r)).collect()
    }
}

impl<T: Wire> Wire for Option<T> {
    fn put(&self, w: &mut Vec<u8>) {
        match self {
            None => w.push(0),
            Some(v) => {
                w.push(1);
                v.put(w);
            }
        }
    }
    fn get(r: &mut Reader) -> Option<Self> {
        match u8::get(r)? {
            0 => Some(None),
            1 => Some(Some(T::get(r)?)),
            _ => None,
        }
    }
}

impl<A: Wire, B: Wire> Wire for (A, B) {
    fn put(&self, w: &mut Vec<u8>) {
        self.0.put(w);
        self.1.put(w);
    }
    fn get(r: &mut Reader) -> Option<Self> {
        Some((A::get(r)?, B::get(r)?))
    }
}

impl<T: Wire, const N: usize> Wire for [T; N] {
    fn put(&self, w: &mut Vec<u8>) {
        for v in self {
            v.put(w);
        }
    }
    fn get(r: &mut Reader) -> Option<Self> {
        let v: Vec<T> = (0..N).map(|_| T::get(r)).collect::<Option<_>>()?;
        v.try_into().ok()
    }
}

// Particle symbols and bubble icons are `&'static` in the API. The app gets them as text and keeps
// one copy of each (there are only a handful), so they can stay `&'static` on its side too.
thread_local! {
    static TEXTS: RefCell<HashMap<String, &'static str>> = RefCell::new(HashMap::new());
    static ICONS: RefCell<HashMap<Vec<String>, &'static [&'static str]>> = RefCell::new(HashMap::new());
}

fn keep_text(s: String) -> &'static str {
    TEXTS.with(|t| *t.borrow_mut().entry(s.clone()).or_insert_with(|| Box::leak(s.into_boxed_str())))
}

impl Wire for &'static str {
    fn put(&self, w: &mut Vec<u8>) {
        self.to_string().put(w);
    }
    fn get(r: &mut Reader) -> Option<Self> {
        let s = String::get(r)?;
        (s.len() <= 64).then(|| keep_text(s))
    }
}

impl Wire for &'static [&'static str] {
    fn put(&self, w: &mut Vec<u8>) {
        self.iter().map(|s| s.to_string()).collect::<Vec<_>>().put(w);
    }
    fn get(r: &mut Reader) -> Option<Self> {
        let rows = Vec::<String>::get(r)?;
        if rows.len() > 32 {
            return None;
        }
        Some(ICONS.with(|icons| {
            *icons.borrow_mut().entry(rows.clone()).or_insert_with(|| {
                let v: Vec<&'static str> = rows.into_iter().map(keep_text).collect();
                Box::leak(v.into_boxed_slice())
            })
        }))
    }
}

// MARK: The boundary types

/// `Wire` for a plain struct, field by field.
macro_rules! fields {
    ($t:ident { $($f:ident),* $(,)? }) => {
        impl Wire for $t {
            fn put(&self, w: &mut Vec<u8>) {
                $(self.$f.put(w);)*
            }
            fn get(r: &mut Reader) -> Option<Self> {
                Some($t { $($f: Wire::get(r)?),* })
            }
        }
    };
}

fields!(Pt { x, y });
fields!(Color { r, g, b, a });
fields!(Prop { rows, col, lift });
fields!(FreeSprite { rows, x, y, px, own_colors });
fields!(Twin { frame, center_x, scale, y_offset, facing_left });
fields!(Look { frame, palette, y_offset, scale, props, free, back, x_offset, twin, beam });
fields!(Particle { text, color, size, x, y, vx, vy, life, max_life, ay });
fields!(PictureFrame { rows, anchor, px });
fields!(Input { dt, now, idle, key_idle, mouse });
#[rustfmt::skip]
fields!(Render {
    look, picture, prop_shift, paw, pet_x, min_pet_x, max_pet_x, facing_left, pet_rect, head, width, range,
    dragging, show_pen, particles, bubble, can_remind, home_moved,
});
#[rustfmt::skip]
fields!(About {
    id, name, short_name, category, description, author, version, tricks, sweeps, has_pastime,
    click_trick, right_click_trick, tallest, icon,
});

impl Wire for (f64, f64, f64, f64) {
    fn put(&self, w: &mut Vec<u8>) {
        [self.0, self.1, self.2, self.3].put(w);
    }
    fn get(r: &mut Reader) -> Option<Self> {
        let [a, b, c, d] = <[f64; 4]>::get(r)?;
        Some((a, b, c, d))
    }
}

impl Wire for Palette {
    fn put(&self, w: &mut Vec<u8>) {
        self.0.put(w);
    }
    fn get(r: &mut Reader) -> Option<Self> {
        Some(Palette(Wire::get(r)?))
    }
}

impl Wire for Category {
    fn put(&self, w: &mut Vec<u8>) {
        w.push(*self as u8);
    }
    fn get(r: &mut Reader) -> Option<Self> {
        [Category::Critter, Category::Hero, Category::Legend].get(u8::get(r)? as usize).copied()
    }
}

impl Wire for Reminder {
    fn put(&self, w: &mut Vec<u8>) {
        w.push(*self as u8);
    }
    fn get(r: &mut Reader) -> Option<Self> {
        [Reminder::Water, Reminder::Lunch, Reminder::Dinner, Reminder::Stretch].get(u8::get(r)? as usize).copied()
    }
}

impl Wire for Command {
    fn put(&self, w: &mut Vec<u8>) {
        use Command::*;
        match self {
            Pen { width, height, range } => {
                w.push(0);
                [*width, *height, *range].put(w);
            }
            PlaceAt(x) => {
                w.push(1);
                x.put(w);
            }
            Say(text, seconds) => {
                w.push(2);
                text.put(w);
                seconds.put(w);
            }
            ShowPen(seconds) => {
                w.push(3);
                seconds.put(w);
            }
            Trick(i) => {
                w.push(4);
                i.put(w);
            }
            SurpriseTrick => w.push(5),
            Hello => w.push(6),
            Remind(r) => {
                w.push(7);
                r.put(w);
            }
            BeginDrag => w.push(8),
            EndDrag { moved } => {
                w.push(9);
                moved.put(w);
            }
            RightClick => w.push(10),
            StopRoaming => w.push(11),
            HomeSaved => w.push(12),
            DemoWalk => w.push(13),
            DemoTyping(s) => {
                w.push(14);
                s.put(w);
            }
            DemoPastime(s) => {
                w.push(15);
                s.put(w);
            }
            DemoSweep(s) => {
                w.push(16);
                s.put(w);
            }
        }
    }
    fn get(r: &mut Reader) -> Option<Self> {
        use Command::*;
        Some(match u8::get(r)? {
            0 => {
                let [width, height, range] = <[f64; 3]>::get(r)?;
                Pen { width, height, range }
            }
            1 => PlaceAt(f64::get(r)?),
            2 => Say(String::get(r)?, f64::get(r)?),
            3 => ShowPen(f64::get(r)?),
            4 => Trick(usize::get(r)?),
            5 => SurpriseTrick,
            6 => Hello,
            7 => Remind(Reminder::get(r)?),
            8 => BeginDrag,
            9 => EndDrag { moved: bool::get(r)? },
            10 => RightClick,
            11 => StopRoaming,
            12 => HomeSaved,
            13 => DemoWalk,
            14 => DemoTyping(f64::get(r)?),
            15 => DemoPastime(f64::get(r)?),
            16 => DemoSweep(f64::get(r)?),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_round_trip() {
        for c in [
            Command::Pen { width: 1.0, height: 2.0, range: 3.0 },
            Command::Say("hi".into(), 1.5),
            Command::EndDrag { moved: true },
            Command::Remind(Reminder::Dinner),
            Command::DemoSweep(4.0),
        ] {
            let bytes = encode(&c);
            assert_eq!(encode(&decode::<Command>(&bytes).unwrap()), bytes);
        }
    }

    #[test]
    fn short_or_long_input_is_refused() {
        let bytes = encode(&Command::Say("hello".into(), 1.0));
        assert!(decode::<Command>(&bytes[..bytes.len() - 1]).is_none());
        let mut more = bytes.clone();
        more.push(0);
        assert!(decode::<Command>(&more).is_none());
        assert!(decode::<String>(&[255, 255, 255, 255]).is_none());
    }
}
