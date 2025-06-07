use esp_println::println;
use fontdue::{Font, FontSettings};

const FONT_SPACE_MONO: &[u8] = include_bytes!("../fonts/SpaceMono-Numbers.ttf");

pub fn init_font() -> Font {
    println!("Parsing the font");
    let f = Font::from_bytes(FONT_SPACE_MONO, FontSettings::default());
    println!("{:?}", f);
    println!("Result written");
    f.unwrap()
}
