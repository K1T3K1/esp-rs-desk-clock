use esp_println::println;
use fontdue::{Font, FontSettings};

const FONT_NUMBERS: &[u8] = include_bytes!("../fonts/SpaceMono-Numbers.ttf");
const FONT_LETTERS: &[u8] = include_bytes!("../fonts/KarlaLowercase.woff2");

pub fn init_font() -> Font {
    println!("Parsing the font");
    let fnum = Font::from_bytes(FONT_NUMBERS, FontSettings::default());
    println!("Parsed num");
    fnum.unwrap()
}

pub fn init_letters_font() -> Font {
    let f = Font::from_bytes(FONT_LETTERS, FontSettings::default());
    f.unwrap()
}


