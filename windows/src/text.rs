//! Text for the shared canvas, drawn by GDI: white on black with greyscale antialiasing, kept as a coverage mask.

use crate::sys;
use deskpet_host::canvas::{Font, Mask, TextRaster};
use std::collections::HashMap;

pub struct GdiText {
    fonts: HashMap<(Font, i32), sys::Font>,
    scratch: Option<sys::Dib>,
}

impl GdiText {
    pub fn new() -> GdiText {
        GdiText { fonts: HashMap::new(), scratch: None }
    }
}

impl TextRaster for GdiText {
    fn render(&mut self, text: &str, font: Font, px: i32) -> Mask {
        let f = self
            .fonts
            .entry((font, px))
            .or_insert_with(|| match font {
                Font::Symbol => sys::Font::new("Segoe UI Symbol", px, 700),
                Font::Ui => sys::Font::new("Segoe UI", px, 600),
            })
            .handle();
        let (w, h) = sys::text_extent(f, text);
        let (w, h) = (w.max(1), h.max(1));
        let need_new = match &self.scratch {
            Some(d) => d.w < w || d.h < h,
            None => true,
        };
        if need_new {
            self.scratch = Some(sys::Dib::new(w.max(64), h.max(32)));
        }
        let dib = self.scratch.as_mut().unwrap();
        let stride = dib.w;
        let pixels = dib.draw_text(f, text, w, h);
        let mut cov = vec![0u8; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let p = pixels[(y * stride + x) as usize];
                let (r, g, b) = ((p >> 16) & 0xFF, (p >> 8) & 0xFF, p & 0xFF);
                cov[(y * w + x) as usize] = r.max(g).max(b) as u8;
            }
        }
        Mask { w, h, cov }
    }
}
