use sdl2::keyboard::Keycode;

use core::{Emulator, Button};

pub fn handle_keydown(keycode: Keycode, emulator: &mut Emulator) {
    if let Some(button) = map_keycode(keycode) {
        emulator.button_press(button);
    }
}

pub fn handle_keyup(keycode: Keycode, emulator: &mut Emulator) {
    if let Some(button) = map_keycode(keycode) {
        emulator.button_release(button);
    }
}

fn map_keycode(keycode: Keycode) -> Option<Button> {
    Some(match keycode {
        Keycode::C => Button::Coin,
        Keycode::Return => Button::P1Start,
        Keycode::Left => Button::P1Left,
        Keycode::Right => Button::P1Right,
        Keycode::Up | Keycode::Z => Button::P1Shoot,
        Keycode::X => Button::P2Start,
        Keycode::A => Button::P2Left,
        Keycode::D => Button::P2Right,
        Keycode::W | Keycode::Space => Button::P2Shoot,
        _ => return None,
    })
}