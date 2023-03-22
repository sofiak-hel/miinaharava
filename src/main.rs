//! hello

#![deny(clippy::all)]
#![allow(missing_docs)]

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::rwops::RWops;
use sdl2::{event::Event, surface::Surface};
use std::time::Instant;

pub mod minefield;

/// main docs
pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut last = Instant::now();

    let atlas =
        Surface::load_bmp_rw(&mut RWops::from_bytes(include_bytes!("./atlas.bmp")).unwrap())
            .unwrap();
    let texture = texture_creator.create_texture_from_surface(atlas).unwrap();

    'running: loop {
        let now = Instant::now();
        let _delta = (Instant::now() - last).as_secs_f32();
        last = now;

        canvas.set_draw_color(Color::RGB(64, 64, 150));
        canvas.clear();
        canvas
            .copy(
                &texture,
                Rect::new(16, 16, 16, 16),
                Rect::from_center(canvas.viewport().center(), 160, 160),
            )
            .unwrap();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
    }
}
