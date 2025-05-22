#![no_std]
#![no_main]

use esp_clock::{dht::DHT, led_embassy};

use embassy_time::Timer;
use esp_hal::{
    clock::CpuClock,
    gpio::Pin,
    timer::timg::TimerGroup,
};
use esp_println::println;

use embassy_executor::Spawner;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timg0.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let timer_embassy = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_embassy.timer0);

    // PINS
    let led_pin = peripherals.GPIO2;
    let dht_pin = peripherals.GPIO5;

    let dht = DHT::new(dht_pin);

    let mut humidity = 0.0;
    let mut temperature = 0.0;

    while humidity == 0.0 {
        match dht.read_data() {
            Some(d) => {
                humidity = d.humidity;
                temperature = d.temperature;
            }
            None => (),
        }
    }

    spawner
        .spawn(led_embassy::led_task(led_pin.degrade()))
        .unwrap();

    loop {
        Timer::after_secs(2).await;
        match dht.read_data() {
            Some(d) => {
                humidity = (humidity + d.humidity) / 2.0;
                temperature = (temperature + d.temperature) / 2.0;
            }
            None => (),
        }

        println!(
            "Humidity: {:.2}% | Temperature: {:.2}°C",
            humidity, temperature
        );
    }
}
