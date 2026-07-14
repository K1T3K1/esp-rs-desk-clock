use crate::{
    display_utils::{
        DataDisplay, Padding, PaddingData, PaddingLeft, RefreshTextRect,
    },
    xtlcd::XTLCD,
};
use alloc::string::ToString;
use chrono::Timelike;
use esp_hal::rtc_cntl::Rtc;
use fontdue::Font;

extern crate alloc;

pub struct Clock<'a> {
    rtc:       Rtc<'a>,
    x:         u16,
    y:         u16,
    font_size: f32,
    shift:     u16,
    color:     [u8; 2],

    rhour:   RefreshTextRect<u32, PaddingLeft>,
    rminute: RefreshTextRect<u32, PaddingLeft>,
    rsecond: RefreshTextRect<u32, PaddingLeft>,
}

impl<'a> Clock<'a> {
    pub fn new(
        rtc: Rtc<'a>,
        x: u16,
        y: u16,
        font_size: f32,
        color: [u8; 2],
    ) -> Self {
        let shift = (font_size / 4.0) as u16;
        Clock {
            rtc,
            x,
            y,
            font_size,
            shift,
            color,
            rhour: RefreshTextRect::default(),
            rminute: RefreshTextRect::default(),
            rsecond: RefreshTextRect::default(),
        }
    }
}

impl<'a> DataDisplay for Clock<'a> {
    fn init_display(&mut self, display: &mut XTLCD, font: &Font) {
        let background_color = [0x00, 0x00];
        let mut offset = 0;
        let time = self.rtc.current_time().time();
        let mut hour = time.hour().to_string();
        if hour.len() == 1 {
            hour = "0".to_string() + &hour;
        }
        let mut m = time.minute().to_string();
        if m.len() == 1 {
            m = "0".to_string() + &m;
        }
        let mut s = time.second().to_string();
        if s.len() == 1 {
            s = "0".to_string() + &s;
        }

        let (w, h) = display.draw_text(
            self.x + offset,
            self.y,
            &s.to_string(),
            font,
            self.font_size,
            &self.color,
            &background_color,
        );
        self.rsecond = RefreshTextRect {
            control_value:    time.second(),
            x:                self.x + offset,
            y:                self.y,
            text_color:       self.color.clone(),
            background_color: background_color.clone(),
            text_size:        self.font_size,
            padding:          PaddingLeft {
                data: PaddingData {
                    width:  2,
                    symbol: '0',
                },
            },
        };

        offset = offset + w + self.shift;
        let (w, _) = display.draw_text(
            self.x + offset,
            self.y,
            ":",
            font,
            self.font_size,
            &self.color,
            &background_color,
        );

        offset = offset + w + self.shift;
        let (w, h) = display.draw_text(
            self.x + offset,
            self.y,
            &m.to_string(),
            font,
            self.font_size,
            &self.color,
            &background_color,
        );
        self.rminute = RefreshTextRect {
            control_value:    time.minute(),
            x:                self.x + offset,
            y:                self.y,
            text_color:       self.color.clone(),
            background_color: background_color.clone(),
            text_size:        self.font_size,
            padding:          PaddingLeft {
                data: PaddingData {
                    width:  2,
                    symbol: '0',
                },
            },
        };

        offset = offset + w + self.shift;
        let (w, _) = display.draw_text(
            self.x + offset,
            self.y,
            ":",
            font,
            self.font_size,
            &self.color,
            &background_color,
        );

        offset = offset + w + self.shift;
        let (w, h) = display.draw_text(
            self.x + offset,
            self.y,
            &hour.to_string(),
            font,
            self.font_size,
            &self.color,
            &background_color,
        );
        self.rhour = RefreshTextRect {
            control_value:    time.hour(),
            x:                self.x + offset,
            y:                self.y,
            text_color:       self.color.clone(),
            background_color: background_color.clone(),
            text_size:        self.font_size,
            padding:          PaddingLeft {
                data: PaddingData {
                    width:  2,
                    symbol: '0',
                },
            },
        };
    }

    fn refresh_display(&mut self, display: &mut XTLCD, font: &Font) {
        let time = self.rtc.current_time().time();

        self.rhour.refresh_checked(display, font, time.hour());
        self.rminute.refresh_checked(display, font, time.minute());
        self.rsecond.refresh_checked(display, font, time.second());
    }
}
