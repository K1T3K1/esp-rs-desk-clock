use core::fmt::Display;

use esp_println::println;
use fontdue::{Font, FontSettings};

use crate::xtlcd::XTLCD;
extern crate alloc;
use alloc::string::{String, ToString};

const FONT_NUMBERS: &[u8] = include_bytes!("../fonts/share_full_minimal.ttf");
//const FONT_LETTERS: &[u8] = include_bytes!("../fonts/share_min.ttf");

pub fn init_font() -> Font {
    println!("Parsing the font");
    let fnum = Font::from_bytes(
        FONT_NUMBERS,
        FontSettings {
            scale: 22.0,
            load_substitutions: false,
            ..FontSettings::default()
        },
    );
    println!("Parsed num");
    fnum.unwrap()
}

pub trait Padding {
    fn add_pading(&self, text: &mut String) {}
}

#[derive(Default, Clone, Copy)]
pub struct PaddingData {
    pub width:  u8,
    pub symbol: char,
}

#[derive(Default, Clone, Copy)]
pub struct PaddingLeft {
    pub data: PaddingData,
}

#[derive(Default, Clone, Copy)]
pub struct PaddingRight {
    pub data: PaddingData,
}

#[derive(Default, Clone, Copy)]
pub struct PaddingNone;

impl Padding for PaddingLeft {
    fn add_pading(&self, text: &mut String) {
        let pad_width = self.data.width as usize;
        let text_len = text.len();
        if pad_width == 0 || text_len >= pad_width {
            return;
        }

        // Calculate how much padding we can actually add
        let padding_to_add = pad_width - text_len;

        let padding = (0..padding_to_add)
            .map(|_| self.data.symbol)
            .collect::<String>();
        *text = padding + text;
    }
}
impl Padding for PaddingRight {}
impl Padding for PaddingNone {}

#[derive(Default, Copy, Clone)]
pub struct RefreshTextRect<T: PartialEq + ToString, P: Padding> {
    // At the moment only single color background
    // Would need mut buffer<'a> to be able to hold a background read from SPI
    pub control_value:    T,
    pub x:                u16,
    pub y:                u16,
    pub text_color:       [u8; 2],
    pub background_color: [u8; 2],
    pub text_size:        f32,
    pub padding:          P,
}

impl<T, P> RefreshTextRect<T, P>
where
    T: PartialEq + ToString,
    P: Padding,
{
    #[inline]
    pub fn refresh_checked(
        &mut self,
        display: &mut XTLCD,
        font: &Font,
        value: T,
    ) where
        T: Display,
    {
        if self.control_value != value {
            self.refresh_unchecked(display, font, &value);
            self.control_value = value;
        }
    }

    #[inline]
    pub fn refresh_unchecked(
        &mut self,
        display: &mut XTLCD,
        font: &Font,
        value: &T,
    ) {
        let text = &mut value.to_string();
        self.padding.add_pading(text);
        display.draw_text(
            self.x,
            self.y,
            &text,
            font,
            self.text_size,
            &self.text_color,
            &self.background_color,
        );
    }
}

pub trait DataDisplay {
    fn init_display(&mut self, display: &mut XTLCD, font: &Font);
    fn refresh_display(&mut self, display: &mut XTLCD, font: &Font);
}
