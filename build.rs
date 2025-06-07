use std::{fs, path::Path};

fn main() {
    linker_be_nice();
    get_images();
    // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

fn get_images() {
    let images_dir = Path::new("images");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("images.rs");

    let mut output = String::new();
    output.push_str("// Auto-generated image data for ST7796\n");
    output.push_str("pub struct ImageData {\n");
    output.push_str("    pub width: u16,\n");
    output.push_str("    pub height: u16,\n");
    output.push_str("    pub data: &'static [u8], // RGB565 as bytes\n");
    output.push_str("}\n\n");

    if images_dir.exists() {
        for entry in fs::read_dir(images_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().map_or(false, |ext| {
                ext == "png" || ext == "jpg" || ext == "jpeg"
            }) {
                let name = path.file_stem().unwrap().to_str().unwrap();

                let img = image::open(&path).unwrap().fliph();
                let rgb8 = img.to_rgb8();
                let (width, height) = rgb8.dimensions();

                let mut rgb565_bytes = Vec::new();
                for x in 0..width {
                    for y in 0..height {
                        let pixel = rgb8.get_pixel(x, y);
                        let r = pixel[0];
                        let g = pixel[1];
                        let b = pixel[2];

                        let r5 = (r >> 3) as u16;
                        let g6 = (g >> 2) as u16;
                        let b5 = (b >> 3) as u16;
                        let rgb565 = (r5 << 11) | (g6 << 5) | b5;

                        rgb565_bytes.push((rgb565 >> 8) as u8);
                        rgb565_bytes.push((rgb565 & 0xFF) as u8);
                    }
                }

                output.push_str(&format!(
                    "pub const {}: ImageData = ImageData {{\n",
                    name.to_uppercase().replace("-", "_")
                ));
                output.push_str(&format!("    width: {},\n", width));
                output.push_str(&format!("    height: {},\n", height));
                output.push_str("    data: &[\n");

                for (i, byte) in rgb565_bytes.iter().enumerate() {
                    if i % 16 == 0 {
                        output.push_str("        ");
                    }
                    output.push_str(&format!("0x{:02x}, ", byte));
                    if i % 16 == 15 {
                        output.push_str("\n");
                    }
                }
                if rgb565_bytes.len() % 16 != 0 {
                    output.push_str("\n");
                }

                output.push_str("    ],\n};\n\n");

                println!("Processed image: {} ({}x{})", name, width, height);
            }
        }
    } else {
        output.push_str("// No images directory found\n");
    }

    fs::write(&dest_path, output).unwrap();
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                "_defmt_timestamp" => {
                    eprintln!();
                    eprintln!("💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`");
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=-Wl,--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}
