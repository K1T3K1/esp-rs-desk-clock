use embassy_time::Timer;
use esp_hal::gpio::{AnyPin, Level, Output, OutputConfig};

#[embassy_executor::task]
pub async fn led_task(pin: AnyPin) {
    let mut led = Output::new(pin, Level::Low, OutputConfig::default());

    loop {
        led.set_high();
        Timer::after_secs(1).await;
        led.set_low();
        Timer::after_secs(1).await;
    }
}
