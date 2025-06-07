// LCD_CS -> GPIO13
// LCD_RST -> GPIO12
// LCD_RS D/C -> GPIO14
// SDI MOSI -> GPIO23 V_SPI_D MOSI
// SCK -> GPIO18 V_SPI_CLK SCK
// let mut spi = Spi::new(
//     peripherals.SPI2,
//     Config::default().with_frequency(Rate::from_khz(100)).
// with_mode(Mode::_0) )?
// .with_sck(peripherals.GPIO0)
// .with_mosi(peripherals.GPIO1)
// .with_miso(peripherals.GPIO2);

use esp_hal::{
    delay::Delay,
    gpio::{GpioPin, InputPin, Level, Output, OutputConfig, OutputPin},
    peripheral::Peripheral,
    peripherals::Peripherals,
    spi::master::{Config, ConfigError, Spi},
    time::Rate,
    Blocking,
};
use fontdue::Font;

use crate::ImageData;

mod commands {
    pub const NOP: u8 = 0x00;
    pub const SWRESET: u8 = 0x01;

    pub const RDDID: u8 = 0x04; // ReadDisplayID
    pub const RNOEDSI: u8 = 0x05; // ReadNumberoftheErrorsonDSI
    pub const RDDST: u8 = 0x09; // ReadDisplayStatus
    pub const RDDPM: u8 = 0x0A; // ReadDisplayPowerMode
    pub const RDDMADCTL: u8 = 0x0B; // ReadDisplayMADCTL
    pub const RDDCOLMOD: u8 = 0x0C; // ReadDisplayPixelFormat
    pub const RDDIM: u8 = 0x0D; // ReadDisplayImageMode
    pub const RDDSM: u8 = 0x0E; // ReadDisplaySignalMode
    pub const RDDSDR: u8 = 0x0F; // ReadDisplaySelfDiagnosticResult
    pub const SLPIN: u8 = 0x10; // SleepIn
    pub const SLPOUT: u8 = 0x11; // SleepOut
    pub const PTLON: u8 = 0x12; // PartialDisplayModeOn
    pub const NORON: u8 = 0x13; // NormalDisplayModeOn
    pub const INVOFF: u8 = 0x20; // InversionOff
    pub const INVON: u8 = 0x21; // InversionOn
    pub const DISPON: u8 = 0x29; // Display On
    pub const CASET: u8 = 0x2A;
    pub const RASET: u8 = 0x2B;
    pub const RAMWR: u8 = 0x2C;
    pub const MADCTL: u8 = 0x36; // MemoryDataAccessControl
    pub const COLMOD: u8 = 0x3A; // InterfacePixelFormat
}

pub mod enums {
    #[repr(u8)]
    pub enum COLMODS {
        NOMOD = 0x00,
        RGB565 = 0x50,
        RGB666 = 0x60,
        SPI16 = 0x05,
        SPI18 = 0x06,
        SPI24 = 0x07,
    }

    #[repr(u8)]
    pub enum MadCtlRWOrder {
        Row = 0x80,      // MY = 1
        Column = 0x40,   // MX = 1
        Exchange = 0x20, // MV = 1
    }

    #[repr(u8)]
    pub enum MadCtlVerticalRefreshOrder {
        TopToBottom = 0x00, // ML = 0
        BottomToTop = 0x10, // ML = 1
    }

    #[repr(u8)]
    pub enum MadCtlColorOrder {
        RGB = 0x00, // RGB = 0
        BGR = 0x08, // RGB = 1
    }

    #[repr(u8)]
    pub enum MadCtlHorizontalRefreshOrder {
        LeftToRight = 0x00, // MH = 0
        RightToLeft = 0x04, // MH = 1
    }
}

pub struct XTLCD<'a> {
    reset_pin:   Output<'a>,
    command_pin: Output<'a>,
    spi:         Spi<'a, Blocking>,
}

impl<'a> XTLCD<'a> {
    pub fn new<
        const SELECT_PIN: u8,
        const RESET_PIN: u8,
        const COMMAND_PIN: u8,
        const SDI_PIN: u8,
        const SCK_PIN: u8,
        const SDO_PIN: u8,
    >(
        select_pin: GpioPin<SELECT_PIN>,
        reset_pin: GpioPin<RESET_PIN>,
        command_pin: GpioPin<COMMAND_PIN>,
        sdi_pin: GpioPin<SDI_PIN>,
        sck_pin: GpioPin<SCK_PIN>,
        sdo_pin: GpioPin<SDO_PIN>,
        peripherals: &Peripherals,
    ) -> Result<Self, ConfigError>
    where
        GpioPin<SELECT_PIN>: OutputPin,
        GpioPin<RESET_PIN>: OutputPin,
        GpioPin<COMMAND_PIN>: OutputPin,
        GpioPin<SDI_PIN>: OutputPin,
        GpioPin<SCK_PIN>: OutputPin,
        GpioPin<SDO_PIN>: InputPin,
    {
        let spi_num = unsafe { peripherals.SPI2.clone_unchecked() };
        let spi = Spi::new(
            spi_num,
            Config::default()
                .with_frequency(Rate::from_mhz(20))
                .with_mode(esp_hal::spi::Mode::_0),
        )?
        .with_sck(sck_pin)
        .with_mosi(sdi_pin)
        .with_miso(sdo_pin)
        .with_cs(select_pin);

        Ok(XTLCD {
            reset_pin: Output::new(
                reset_pin,
                Level::Low,
                OutputConfig::default(),
            ),
            command_pin: Output::new(
                command_pin,
                Level::Low,
                OutputConfig::default(),
            ),
            spi,
        })
    }

    pub fn with_init(&mut self) -> &mut Self {
        let delay = Delay::new();
        self.hardware_reset();
        delay.delay_millis(150);
        self.write_command(commands::SWRESET);
        delay.delay_millis(150);
        self
    }

    pub fn with_sleep_out(&mut self) -> &mut Self {
        let delay = Delay::new();
        self.write_command(commands::SLPOUT);
        delay.delay_millis(120);
        self
    }

    pub fn with_normal_display_on(&mut self) -> &mut Self {
        self.write_command(commands::NORON);
        self.write_command(commands::DISPON);
        self
    }

    #[inline]
    pub fn with_column_boundaries(
        &mut self,
        leftmost: u16,
        rightmost: u16,
    ) -> &mut Self {
        self.write_command(commands::RASET); // For some reason these instructions work opposite to
                                             // instruction
        let left = leftmost.to_be_bytes();
        let right = rightmost.to_be_bytes();
        self.write_data(&[left[0], left[1], right[0], right[1]]);
        self
    }

    #[inline]
    pub fn with_row_boundaries(
        &mut self,
        bottommost: u16,
        topmost: u16,
    ) -> &mut Self {
        self.write_command(commands::CASET); // For some reason these instructions work opposite to
                                             // instruction
        let left = bottommost.to_be_bytes();
        let right = topmost.to_be_bytes();
        self.write_data(&[left[0], left[1], right[0], right[1]]);
        self
    }

    pub fn with_interface_pixel_format(
        &mut self,
        rgb: enums::COLMODS,
        spi: enums::COLMODS,
    ) -> &mut Self {
        self.write_command(commands::COLMOD);
        self.write_data(&[rgb as u8 | spi as u8]);
        self
    }

    pub fn with_memory_data_access_modes(
        &mut self,
        wr_order: enums::MadCtlRWOrder,
        vertical_refresh_order: enums::MadCtlVerticalRefreshOrder,
        rgb_order: enums::MadCtlColorOrder,
        horizontal_refresh_order: enums::MadCtlHorizontalRefreshOrder,
    ) -> &mut Self {
        self.write_command(commands::MADCTL);
        self.write_data(&[wr_order as u8
            | vertical_refresh_order as u8
            | rgb_order as u8
            | horizontal_refresh_order as u8]);
        self
    }

    pub fn with_inversion(&mut self, is_inverted: bool) -> &mut Self {
        self.write_command(if is_inverted {
            commands::INVON
        } else {
            commands::INVOFF
        });
        self
    }

    pub fn with_background(&mut self) -> &mut Self {
        self.with_column_boundaries(0, 479)
            .with_row_boundaries(0, 319);
        self.write_command(commands::RAMWR);
        for _ in 0..(480 * 320) {
            self.write_data(&[0x00, 0x00]);
        }
        self
    }

    fn hardware_reset(&mut self) {
        self.reset_pin.set_high();
        let delay = Delay::new();
        delay.delay_millis(10);
        self.reset_pin.set_low();
        delay.delay_millis(10);
        self.reset_pin.set_high();
        delay.delay_millis(120);
    }

    #[inline]
    pub fn write_command(&mut self, command: u8) {
        self.command_pin.set_low();
        let _ = self.spi.write(&[command]);
    }

    #[inline]
    pub fn write_data(&mut self, data: &[u8]) {
        self.command_pin.set_high();
        let _ = self.spi.write(data);
    }

    #[inline]
    pub fn draw_rect(
        &mut self,
        x: u16,
        y: u16,
        height: u16,
        width: u16,
        color: &[u8; 2],
    ) {
        self.with_row_boundaries(y, y + height - 1)
            .with_column_boundaries(x, x + width - 1);

        self.write_command(commands::RAMWR);
        for _ in 0..(height * width) {
            self.write_data(color);
        }
    }

    pub fn draw_image(&mut self, x: u16, y: u16, image: &ImageData) {
        const CHUNK_SIZE: usize = 1024;

        self.with_row_boundaries(y, y + image.height - 1)
            .with_column_boundaries(x, x + image.width - 1);

        self.write_command(commands::RAMWR);
        for chunk in image.data.chunks(CHUNK_SIZE) {
            self.write_data(chunk);
        }
    }

    #[inline]
    pub fn draw_text(
        &mut self,
        x: u16,
        y: u16,
        text: &str,
        font: &Font,
        size: f32,
        color: &[u8; 2],
    ) -> (u16, u16) {
        let mut x_offset = 0;
        let mut height = 0;

        for letter in text.chars().rev() {
            let (metrics, bitmap) = font.rasterize(letter, size);
            if metrics.height > height {
                height = metrics.height;
            }

            // Skip empty characters
            if metrics.width == 0 || metrics.height == 0 {
                x_offset += metrics.advance_width as u16;
                continue;
            }

            // Draw each pixel of the character
            for row in 0..metrics.height {
                for col in (0..metrics.width).rev() {
                    let bitmap_index = row * metrics.width + col;
                    if bitmap_index < bitmap.len() {
                        let alpha = bitmap[bitmap_index];

                        // Only draw opaque pixels
                        if alpha > 128 {
                            let pixel_x = (x + x_offset - (col as u16)) + (metrics.width as u16);
                            let pixel_y = y + row as u16;

                            // Draw single pixel
                            self.with_row_boundaries(pixel_y, pixel_y)
                                .with_column_boundaries(pixel_x, pixel_x);
                            self.write_command(commands::RAMWR);
                            self.write_data(color);
                        }
                    }
                }
            }

            x_offset += metrics.advance_width as u16;
        }
        (x_offset, height as u16)
    }
}
