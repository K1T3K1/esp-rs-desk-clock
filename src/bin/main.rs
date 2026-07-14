#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use alloc::string::ToString;
use core::intrinsics;
use esp_clock::{
    clock::Clock,
    dht::DHT,
    display_utils::{init_font, DataDisplay},
    xtlcd::{enums, XTLCD},
    IMG_3814,
};

use esp_hal::{
    clock::CpuClock, delay::Delay, main, peripheral::Peripheral, rtc_cntl::Rtc,
    time::Instant, timer::timg::TimerGroup,
};
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 82 * 1024);

    let (
        dht_pin,
        xtlcd_select_pin,
        xtlcd_reset_pin,
        xtlcd_command_pin,
        xtlcd_sdi_pin,
        xtlcd_sck_pin,
        xtlcd_sdo_pin,
        timg0,
        radio_clk,
        rng,
        lpwr,
    ) = unsafe {
        let dht_pin = peripherals.GPIO5.clone_unchecked();

        // Display
        let xtlcd_select_pin = peripherals.GPIO13.clone_unchecked();
        let xtlcd_reset_pin = peripherals.GPIO12.clone_unchecked();
        let xtlcd_command_pin = peripherals.GPIO14.clone_unchecked();
        let xtlcd_sdi_pin = peripherals.GPIO23.clone_unchecked();
        let xtlcd_sck_pin = peripherals.GPIO18.clone_unchecked();
        let xtlcd_sdo_pin = peripherals.GPIO19.clone_unchecked();
        let timg0 = peripherals.TIMG0.clone_unchecked();
        let radio_clk = peripherals.RADIO_CLK.clone_unchecked();
        let rng = peripherals.RNG.clone_unchecked();
        let lpwr = peripherals.LPWR.clone_unchecked();

        (
            dht_pin,
            xtlcd_select_pin,
            xtlcd_reset_pin,
            xtlcd_command_pin,
            xtlcd_sdi_pin,
            xtlcd_sck_pin,
            xtlcd_sdo_pin,
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

    let mut dht = DHT::new(dht_pin, 300, 45, 40.0, [0xFF, 0xFF]);
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

    let mut clock = Clock::new(rtc, 35, 45, 40.0, [0xFF, 0xFF]);

    println!("Initializing font...");
    let fontnum = init_font();

    println!("Font initialized... Writing text");

    clock.init_display(xtlcd, &fontnum);
    dht.init_display(xtlcd, &fontnum);
    dht.read_data();
    dht.refresh_display(xtlcd, &fontnum);
    let delay = Delay::new();
    let mut inst_timer = Instant::now();
    xtlcd.draw_image(20, 150, &IMG_3814);

    loop {
        dht.read_data();
        clock.refresh_display(xtlcd, &fontnum);
        if inst_timer.elapsed().as_secs() > 10 {
            dht.refresh_display(xtlcd, &fontnum);
            inst_timer = Instant::now();
        }
        delay.delay_millis(999);
    }
}
