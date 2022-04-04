pub mod input;
pub mod audio;

use sdl2::keyboard::Mod;
use sdl2::pixels::Color;

pub const WIDTH: u32 = 224;
pub const HEIGHT: u32 = 256;

pub fn has_ctrl(keymod: Mod) -> bool {
    keymod.contains(Mod::RCTRLMOD) || keymod.contains(Mod::LCTRLMOD)
}

pub fn update_pixel_data(pixel_data: &mut [u8], video_ram: &[u8]) -> bool {
    let mut update = false;

    for (b, byte) in video_ram.iter().enumerate() {
        let offset = b * 8;

        for bit in 0..8 {
            let full_index = offset + bit;
            let data_index = full_index * 3;

            let color = if byte & (1 << bit) == 0 {
                Color::BLACK
            } else {
                let x = full_index as u32 / HEIGHT;
                let y = HEIGHT - (full_index as u32 % HEIGHT);
                match_pixel_color(x, y)
            };

            let (r, g, b) = color.rgb();

            if pixel_data[data_index] != r || pixel_data[data_index + 1] != g || pixel_data[data_index + 2] != b {
                pixel_data[data_index + 0] = r;
                pixel_data[data_index + 1] = g;
                pixel_data[data_index + 2] = b;
                update = true;
            }
        }
    }

    update
}

pub fn match_pixel_color(x: u32, y: u32) -> Color {
    match y {
        33..=64 => Color::RED,
        185..=240 => Color::GREEN,
        241..=HEIGHT if x > 16 && x <= 134 => Color::GREEN,
        _ => Color::WHITE,
    }
}
