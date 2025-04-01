use brickadia::save::{BrickColor, Color};
use image::Rgb;

pub const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
pub const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
pub const RED: Rgb<u8> = Rgb([255, 0, 0]);
pub const BLUE: Rgb<u8> = Rgb([0, 0, 255]);

pub fn rgb_to_brick(rgb: Rgb<u8>) -> BrickColor {
    BrickColor::Unique(Color {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
        a: 255,
    })
}