#![no_std]
#![no_main]

use esp_clock::{
    dht::DHT,
    led_embassy,
    xtlcd::{enums, XTLCD},
    CULTLEADEROVERWORLD,
};

use embassy_time::Timer;
use esp_hal::{
    clock::CpuClock, gpio::Pin, peripheral::Peripheral, timer::timg::TimerGroup,
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

    let (
        led_pin,
        dht_pin,
        xtlcd_select_pin,
        xtlcd_reset_pin,
        xtlcd_command_pin,
        xtlcd_sdi_pin,
        xtlcd_sck_pin,
        xtlcd_sdo_pin,
        tg,
        timg0,
        radio_clk,
        rng,
    ) = unsafe {
        let led_pin = peripherals.GPIO2.clone_unchecked();
        let dht_pin = peripherals.GPIO5.clone_unchecked();

        // Display
        let xtlcd_select_pin = peripherals.GPIO13.clone_unchecked();
        let xtlcd_reset_pin = peripherals.GPIO12.clone_unchecked();
        let xtlcd_command_pin = peripherals.GPIO14.clone_unchecked();
        let xtlcd_sdi_pin = peripherals.GPIO23.clone_unchecked();
        let xtlcd_sck_pin = peripherals.GPIO18.clone_unchecked();
        let xtlcd_sdo_pin = peripherals.GPIO19.clone_unchecked();
        let tg = peripherals.TIMG1.clone_unchecked();
        let timg0 = peripherals.TIMG0.clone_unchecked();
        let radio_clk = peripherals.RADIO_CLK.clone_unchecked();
        let rng = peripherals.RNG.clone_unchecked();

        (
            led_pin,
            dht_pin,
            xtlcd_select_pin,
            xtlcd_reset_pin,
            xtlcd_command_pin,
            xtlcd_sdi_pin,
            xtlcd_sck_pin,
            xtlcd_sdo_pin,
            tg,
            timg0,
            radio_clk,
            rng,
        )
    };

    let tg0 = TimerGroup::new(timg0);
    let _init =
        esp_wifi::init(tg0.timer0, esp_hal::rng::Rng::new(rng), radio_clk)
            .unwrap();

    let timer_embassy = TimerGroup::new(tg);
    esp_hal_embassy::init(timer_embassy.timer0);

    let dht = DHT::new(dht_pin);
    let mut xtlcd_uninit = XTLCD::new(
        xtlcd_select_pin,
        xtlcd_reset_pin,
        xtlcd_command_pin,
        xtlcd_sdi_pin,
        xtlcd_sck_pin,
        xtlcd_sdo_pin,
        &peripherals,
    )
    .unwrap();
    let xtlcd = xtlcd_uninit
        .with_init()
        .with_sleep_out()
        .with_memory_data_access_modes(
            enums::MadCtlRWOrder::Row,
            enums::MadCtlVerticalRefreshOrder::TopToBottom,
            enums::MadCtlColorOrder::BGR,
            enums::MadCtlHorizontalRefreshOrder::LeftToRight,
        )
        .with_row_boundaries(0, 319)
        .with_column_boundaries(0, 479)
        .with_interface_pixel_format(
            enums::COLMODS::RGB565,
            enums::COLMODS::SPI16,
        )
        .with_inversion(true)
        .with_normal_display_on()
        .with_background();

    Timer::after_millis(150).await;

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

    Timer::after_millis(150).await;

    xtlcd.draw_image(100, 100, &CULTLEADEROVERWORLD);
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
