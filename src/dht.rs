use esp_hal::{
    gpio::{
        interconnect::{InputSignal, OutputSignal},
        GpioPin, InputPin, Level, Output, OutputConfig, OutputPin,
    },
    time::{Duration, Instant},
};
use esp_println::println;

pub struct DHT {
    input:  InputSignal,
    output: OutputSignal,
}

#[derive(Debug)]
pub struct DHTData {
    pub humidity:    f32,
    pub temperature: f32,
}

impl DHT {
    pub fn new<const GPIONUM: u8>(pin_num: GpioPin<GPIONUM>) -> Self
    where
        GpioPin<GPIONUM>: InputPin + OutputPin,
    {
        let pin = Output::new(
            pin_num,
            Level::High,
            OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
        );
        let (input, output) = pin.split();
        DHT { input, output }
    }

    #[inline]
    fn wait_for(&self, level: Level) -> Duration {
        let timer = Instant::now();
        while self.input.level() != level {}
        return timer.elapsed();
    }

    #[inline]
    fn get_bit(&self, d: Duration) -> u8 {
        match d {
            d if d < Duration::from_micros(32) => 0,
            d if d < Duration::from_micros(70) => 1,
            _ => 1,
        }
    }

    #[inline]
    fn get_data(&self) -> [u8; 5] {
        let mut result = [0u8; 5];

        for byte_index in 0..5 {
            for bit_index in (0..8).rev() {
                let _ = self.wait_for(Level::High);
                let high_duration = self.wait_for(Level::Low);
                let bit = self.get_bit(high_duration);

                result[byte_index] |= bit << bit_index;
            }
        }

        result
    }

    pub fn read_data(&self) -> Option<DHTData> {
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
            return None;
        }

        Some(DHTData {
            humidity:    data[0] as f32 + (data[1] as f32 / 10.0),
            temperature: data[2] as f32 + (data[3] as f32 / 10.0),
        })
    }
}
