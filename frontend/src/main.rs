use std::time::Duration;
use colored::Colorize;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};

use core::CPU;

const WINDOW_TITLE: &str = "Space Invaders";
const WIDTH: u32 = 256;
const HEIGHT: u32 = 224;
const WINDOW_SIZE: f32 = 2.5;
const FPS: u32 = 60;

fn main() {
    let program = include_bytes!("../invaders");

    run(program).unwrap_or_else(|e| {
        println!("{} {}", "Error:".red().bold(), e.to_string().red())
    });
}

fn run(program: &[u8; 0x2000]) -> Result<(), String> {
    let sdl_context = sdl2::init().expect("could not initialize sdl2");
    let video_subsystem = sdl_context.video().expect("could not initialize video system");
    let window = video_subsystem
        .window("Space Invaders", (WIDTH as f32 * WINDOW_SIZE) as u32, (HEIGHT as f32 * WINDOW_SIZE) as u32)
        .position_centered()
        .build().expect("could not build window");

    let mut canvas = window.into_canvas().present_vsync().build().expect("could not build renderer");
    let mut event_pump = sdl_context.event_pump().expect("could not initialize event pump");

    canvas.set_scale(WINDOW_SIZE, WINDOW_SIZE)?;
    canvas.present();

    let mut cpu = CPU::new(program);
    let mut paused = false;

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode: Some(keycode), keymod, .. }
                if keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD) => {
                    match keycode {
                        Keycode::Q => break 'main,
                        Keycode::R => cpu.reset(),
                        _ => {}
                    };
                }
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Escape => {
                            paused = !paused;

                            let title = if paused {
                                format!("Paused · {}", WINDOW_TITLE)
                            } else {
                                WINDOW_TITLE.into()
                            };
                            canvas.window_mut().set_title(&title).ok();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        for _ in 0..35 {
            cpu.step().map_err(|e| e.to_string())?;
        }

        spin_sleep::sleep(Duration::new(0, 1_000_000_000 / FPS));
    }

    Ok(())
}
