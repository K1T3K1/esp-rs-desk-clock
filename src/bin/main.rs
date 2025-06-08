#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use alloc::string::ToString;
use core::intrinsics;
use esp_clock::{
    clock::Clock,
    dht::DHT,
    fonts::{init_font, init_letters_font},
    led_embassy,
    xtlcd::{enums, XTLCD},
};

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::{
    clock::CpuClock, gpio::Pin, peripheral::Peripheral, rtc_cntl::Rtc,
    timer::timg::TimerGroup,
};
use esp_println::println;

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
        lpwr,
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
        let lpwr = peripherals.LPWR.clone_unchecked();

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
            lpwr,
        )
    };

    let tg0 = TimerGroup::new(timg0);
    let _init =
        esp_wifi::init(tg0.timer0, esp_hal::rng::Rng::new(rng), radio_clk)
            .unwrap();

    let rtc = Rtc::new(lpwr);

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
            enums::MadCtlRWOrder::Column,
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

    let mut clock = Clock::new(rtc, 180, 110, 32.0, [0xFF, 0xFF]);

    Timer::after_millis(150).await;
    println!("Initializing font...");
    let fontnum = init_font();
    let fontnum = init_letters_font();

    println!("Font initialized... Writing text");

    clock.draw_time(xtlcd, &fontnum);
    loop {
        Timer::after_secs(1).await;
        match dht.read_data() {
            Some(d) => {
                humidity = unsafe {
                    intrinsics::truncf32((humidity + d.humidity) / 2.0 * 100.0)
                        / 100.0
                };
                temperature = unsafe {
                    intrinsics::truncf32(
                        (temperature + d.temperature) / 2.0 * 100.0,
                    ) / 100.0
                };
            }
            None => (),
        }
        clock.refresh_time(xtlcd, &fontnum);

        xtlcd.draw_rect(50, 160, 50, 300, &[0x00, 0x00]);
        xtlcd.draw_text(
            50,
            160,
            &(humidity.to_string()
                + "%"
                + "..."
                + &temperature.to_string()
                + "°"),
            &fontnum,
            32.0,
            &[0xFF, 0xFF],
        );
    }
}
