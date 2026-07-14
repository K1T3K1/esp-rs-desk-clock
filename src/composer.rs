extern crate alloc;

use alloc::vec::Vec;

pub trait Composer {
    fn redraw_all(&self);
    fn refresh(&self);
    fn update_image(&self);
    fn add_image(&self) -> u8;
    fn add_animation(&self) -> u8;
}

pub trait Animation {
    fn start(&self);
}

trait Animated {
    fn next_frame(&self, prev: u8, next: u8);
}

pub struct XTLCDAnimation {
    images: [XTLCDImage; 12],
}

impl Animation for XTLCDAnimation {}

impl Animated for XTLCDAnimation {
    fn next_frame(&self, prev: u8, next: u8) {}
}

pub struct PixelData {
    color:       [u8; 2],
    has_changed: bool,
}

pub trait Image {}
pub struct XTLCDImage {
    pixel_data: Vec<Vec<PixelData>>,
}

impl Image for XTLCDImage {}

pub struct XTLCDComposer {
    animations: [XTLCDAnimation; 8],
    images:     [XTLCDImage; 64],
}

impl Composer for XTLCDComposer {}
