use core::intrinsics::{self, likely, unlikely};

use esp_hal::{
    gpio::{
        interconnect::{InputSignal, OutputSignal},
        GpioPin, InputPin, Level, Output, OutputConfig, OutputPin,
    },
    time::{Duration, Instant},
};
use esp_println::println;

use crate::display_utils::{
    DataDisplay, PaddingData, PaddingLeft, PaddingNone, RefreshTextRect,
};

extern crate alloc;
use alloc::string::ToString;

pub struct DHT {
    input:           InputSignal,
    output:          OutputSignal,
    pub latest_data: DHTData,
    pub averaged:    DHTData,
    rhum_int:        RefreshTextRect<u8, PaddingLeft>,
    rhum_frac:       RefreshTextRect<u8, PaddingNone>,
    rtemp_int:       RefreshTextRect<u8, PaddingLeft>,
    rtemp_frac:      RefreshTextRect<u8, PaddingNone>,
    x:               u16,
    y:               u16,
    font_size:       f32,
    text_color:      [u8; 2],
}

#[derive(Debug, Clone, Default)]
pub struct DHTData {
    pub humidity:    f32,
    pub temperature: f32,
}

impl DHT {
    pub fn new<const GPIONUM: u8>(
        pin_num: GpioPin<GPIONUM>,
        x: u16,
        y: u16,
        font_size: f32,
        text_color: [u8; 2],
    ) -> Self
    where
        GpioPin<GPIONUM>: InputPin + OutputPin,
    {
        let pin = Output::new(
            pin_num,
            Level::High,
            OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
        );
        let (input, output) = pin.split();
        DHT {
            input,
            output,
            x,
            y,
            font_size,
            text_color,
            latest_data: DHTData::default(),
            averaged: DHTData::default(),
            rhum_int: RefreshTextRect::default(),
            rhum_frac: RefreshTextRect::default(),
            rtemp_int: RefreshTextRect::default(),
            rtemp_frac: RefreshTextRect::default(),
        }
    }

    #[inline]
    fn wait_for(&self, level: Level) -> Duration {
        let timer = Instant::now();
        loop {
            if likely(self.input.level() == level) {
                break;
            }
        }
        //while self.input.level() != level {}
        return timer.elapsed();
    }

    #[inline]
    fn get_bit(&self, d: u64) -> u8 {
        if d > 50 {
            1
        } else {
            0
        }
    }

    #[inline]
    fn get_data(&self) -> [u8; 5] {
        let mut result = [0u8; 5];

        for byte_index in 0..5 {
            for bit_index in (0..8).rev() {
                let _ = self.wait_for(Level::High);
                let high_duration = self.wait_for(Level::Low);
                let bit = self.get_bit(high_duration.as_micros());

                result[byte_index] |= bit << bit_index;
            }
        }

        result
    }

    pub fn read_data(&mut self) {
        self.output.enable_input(false);
        self.output.enable_output(true);
        let start_signal_timer = Instant::now();
        self.output.set_output_high(false);
        while start_signal_timer.elapsed() < Duration::from_millis(18) {}

        let pull_up_timer = Instant::now();
        self.output.set_output_high(true);
        while pull_up_timer.elapsed() < Duration::from_micros(30) {}

        self.output.enable_input(true);

        self.wait_for(Level::Low);
        self.wait_for(Level::High);
        self.wait_for(Level::Low);

        let data = self.get_data();

        if !(data[4] == (data[0] + data[1] + data[2] + data[3]) & 0xFF) {
            println!("Failed to receive temp");
            return;
        }

        self.latest_data = DHTData {
            humidity:    data[0] as f32 + (data[1] as f32 / 10.0),
            temperature: data[2] as f32 + (data[3] as f32 / 10.0),
        };

        if unlikely(self.averaged.temperature == 0.0) {
            self.averaged = self.latest_data.clone()
        } else {
            unsafe {
                self.averaged = DHTData {
                    humidity:    intrinsics::truncf32(
                        (self.averaged.humidity + self.latest_data.humidity)
                            / 2.0
                            * 100.0,
                    ) / 100.0,
                    temperature: intrinsics::truncf32(
                        (self.averaged.temperature
                            + self.latest_data.temperature)
                            / 2.0
                            * 100.0,
                    ) / 100.0,
                }
            }
        }
    }
}

impl DataDisplay for DHT {
    fn init_display(
        &mut self,
        display: &mut crate::xtlcd::XTLCD,
        font: &fontdue::Font,
    ) {
        let background_color = [0x00, 0x00];
        let mut offset = self.x;
        let (w, _) = display.draw_text(
            offset,
            self.y,
            "C",
            font,
            self.font_size,
            &self.text_color,
            &background_color,
        );
        offset = offset + w;
        let (w, _) = display.draw_text(
            offset,
            self.y,
            &"°",
            font,
            self.font_size,
            &self.text_color,
            &background_color,
        );
        offset = offset + w;
        let (w, h) = display.draw_text(
            offset,
            self.y,
            "0",
            font,
            self.font_size,
            &self.text_color,
            &background_color,
        );
        self.rtemp_frac = RefreshTextRect {
            control_value:    0,
            x:                offset,
            y:                self.y,
            text_color:       self.text_color.clone(),
            background_color: background_color.clone(),
            text_size:        self.font_size,
            padding:          PaddingNone {},
        };
        offset = offset + w;
        let (w, _) = display.draw_text(
            offset,
            self.y,
            ".",
            &font,
            self.font_size,
            &self.text_color,
            &background_color,
        );
        offset = offset + w;
        let (w, h) = display.draw_text(
            offset,
            self.y,
            "00",
            &font,
            self.font_size,
            &self.text_color,
            &background_color,
        );
        self.rtemp_int = RefreshTextRect {
            control_value:    0,
            x:                offset,
            y:                self.y,
            text_color:       self.text_color.clone(),
            background_color: background_color.clone(),
            text_size:        self.font_size,
            padding:          PaddingLeft {
                data: PaddingData {
                    width:  2,
                    symbol: ' ',
                },
            },
        };
        offset = self.x;
        let hum_y = self.y + self.font_size as u16;
        let (w, _) = display.draw_text(
            offset,
            hum_y,
            "%",
            font,
            self.font_size,
            &self.text_color,
            &background_color,
        );
        offset = offset + w;
        let (w, h) = display.draw_text(
            offset,
            hum_y,
            "0",
            font,
            self.font_size,
            &self.text_color,
            &background_color,
        );
        self.rhum_frac = RefreshTextRect {
            control_value:    0,
            x:                offset,
            y:                hum_y,
            text_color:       self.text_color.clone(),
            background_color: background_color.clone(),
            text_size:        self.font_size,
            padding:          PaddingNone {},
        };
        offset = offset + w;
        let (w, _) = display.draw_text(
            offset,
            hum_y,
            ".",
            font,
            self.font_size,
            &self.text_color,
            &background_color,
        );
        offset = offset + w;
        let (w, h) = display.draw_text(
            offset,
            hum_y,
            "00",
            font,
            self.font_size,
            &self.text_color,
            &background_color,
        );
        self.rhum_int = RefreshTextRect {
            control_value:    0,
            x:                offset,
            y:                hum_y,
            text_color:       self.text_color.clone(),
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

    fn refresh_display(
        &mut self,
        display: &mut crate::xtlcd::XTLCD,
        font: &fontdue::Font,
    ) {
        let (int_t, float_t, int_h, float_h): (u8, u8, u8, u8) = unsafe {
            let int_t = intrinsics::truncf32(self.averaged.temperature);
            let int_h = intrinsics::truncf32(self.averaged.humidity);
            let float_t = (self.averaged.temperature - int_t) * 10.0;
            let float_h = (self.averaged.humidity - int_h) * 10.0;
            (
                int_t.to_int_unchecked(),
                float_t.to_int_unchecked(),
                int_h.to_int_unchecked(),
                float_h.to_int_unchecked(),
            )
        };
        if float_t > 0 {
            self.rtemp_int.refresh_checked(display, font, int_t);
            self.rtemp_frac.refresh_checked(display, font, float_t);
        } else {
            self.rtemp_int.refresh_checked(display, font, int_t);
        }
        if float_h > 0 {
            self.rhum_int.refresh_checked(display, font, int_h);
            self.rhum_frac.refresh_checked(display, font, float_h);
        } else {
            self.rhum_int.refresh_checked(display, font, int_h);
        }
    }
}
