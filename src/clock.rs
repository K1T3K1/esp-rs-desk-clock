use crate::xtlcd::XTLCD;
use alloc::string::ToString;
use chrono::Timelike;
use esp_hal::rtc_cntl::Rtc;
use fontdue::Font;
extern crate alloc;

struct TimeRect {
    value:  u32,
    x:      u16,
    y:      u16,
    width:  u16,
    height: u16,
}

pub struct Clock<'a> {
    rtc:       Rtc<'a>,
    x:         u16,
    y:         u16,
    font_size: f32,
    shift:     u16,
    color:     [u8; 2],

    rhour:   TimeRect,
    rminute: TimeRect,
    rsecond: TimeRect,
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
            rhour: TimeRect {
                value:  0,
                x:      0,
                y:      0,
                width:  0,
                height: 0,
            },
            rminute: TimeRect {
                value:  0,
                x:      0,
                y:      0,
                width:  0,
                height: 0,
            },
            rsecond: TimeRect {
                value:  0,
                x:      0,
                y:      0,
                width:  0,
                height: 0,
            },
        }
    }

    pub fn draw_time(&mut self, display: &mut XTLCD, font: &Font) {
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
        );
        self.rsecond = TimeRect {
            value:  time.second(),
            x:      self.x + offset,
            y:      self.y,
            width:  w,
            height: h,
        };

        offset = offset + w;
        let (w, _) = display.draw_text(
            self.x + offset,
            self.y + (self.shift / 2),
            ":",
            font,
            self.font_size,
            &self.color,
        );

        offset = offset + w - self.shift;
        let (w, h) = display.draw_text(
            self.x + offset,
            self.y,
            &m.to_string(),
            font,
            self.font_size,
            &self.color,
        );
        self.rminute = TimeRect {
            value:  time.minute(),
            x:      self.x + offset,
            y:      self.y,
            width:  w,
            height: h,
        };

        offset = offset + w;
        let (w, _) = display.draw_text(
            self.x + offset,
            self.y + (self.shift / 2),
            ":",
            font,
            self.font_size,
            &self.color,
        );

        offset = offset + w - self.shift;
        let (w, h) = display.draw_text(
            self.x + offset,
            self.y,
            &hour.to_string(),
            font,
            self.font_size,
            &self.color,
        );
        self.rhour = TimeRect {
            value:  time.hour(),
            x:      self.x + offset,
            y:      self.y,
            width:  w,
            height: h,
        };
    }

    pub fn refresh_time(&mut self, display: &mut XTLCD, font: &Font) {
        let time = self.rtc.current_time().time();

        let mut hour = time.hour().to_string();
        if time.hour() != self.rhour.value {
            if hour.len() == 1 {
                hour = "0".to_string() + &hour;
            }
            display.draw_rect(
                self.rhour.x,
                self.rhour.y,
                self.rhour.height,
                self.rhour.width,
                &[0x00, 0x00],
            );
            display.draw_text(
                self.rhour.x,
                self.rhour.y,
                &hour,
                font,
                self.font_size,
                &self.color,
            );
            self.rhour.value = time.hour();
        }

        let mut minute = time.minute().to_string();
        if time.minute() != self.rminute.value {
            if minute.len() == 1 {
                minute = "0".to_string() + &minute;
            }
            display.draw_rect(
                self.rminute.x,
                self.rminute.y,
                self.rminute.height,
                self.rminute.width,
                &[0x00, 0x00],
            );
            display.draw_text(
                self.rminute.x,
                self.rminute.y,
                &minute,
                font,
                self.font_size,
                &self.color,
            );
            self.rminute.value = time.minute();
        }

        let mut second = time.second().to_string();
        if time.second() != self.rsecond.value {
            if second.len() == 1 {
                second = "0".to_string() + &second;
            }
            display.draw_rect(
                self.rsecond.x,
                self.rsecond.y,
                self.rsecond.height,
                self.rsecond.width,
                &[0x00, 0x00],
            );
            display.draw_text(
                self.rsecond.x,
                self.rsecond.y,
                &second,
                font,
                self.font_size,
                &self.color,
            );
            self.rsecond.value = time.second();
        }
    }
}
