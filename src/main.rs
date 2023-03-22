//! hello

#![deny(clippy::all)]
#![allow(missing_docs)]

use minefield::{Coord, Minefield};
use renderer::Renderer;
use sdl2::keyboard::Keycode;
use sdl2::mouse::{self, MouseButton};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::rwops::RWops;
use sdl2::{event::Event, surface::Surface};
use std::time::Instant;

pub mod minefield;
pub mod renderer;

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

    const size: (usize, usize) = (10, 10);
    let mut minefield = Minefield::<{ size.0 }, { size.1 }>::generate(10);
    let renderer = Renderer::init(&texture_creator);
    let w = 800 / size.0;
    let h = 600 / size.1;
    let scale = w.min(h);
    let target = Rect::from_center(
        canvas.viewport().center(),
        (size.0 * scale) as u32,
        (size.1 * scale) as u32,
    );

    'running: loop {
        let now = Instant::now();
        let _delta = (Instant::now() - last).as_secs_f32();
        last = now;

        canvas.set_draw_color(Color::RGB(64, 64, 150));
        canvas.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => match mouse_btn {
                    MouseButton::Left => {
                        if let Some(coord) = Renderer::get_coord(target, (x, y)) {
                            minefield.reveal(coord).unwrap();
                        }
                    }
                    MouseButton::Right => {
                        if let Some(coord) = Renderer::get_coord(target, (x, y)) {
                            minefield.flag(coord).unwrap();
                        }
                    }
                    _ => {}
                },

                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        let mouse_state = event_pump.mouse_state();
        let drag_pos = if mouse_state.left() || mouse_state.right() {
            Some((mouse_state.x(), mouse_state.y()))
        } else {
            None
        };

        renderer.draw(&minefield, &mut canvas, target, drag_pos);
        // The rest of the game loop goes here...

        canvas.present();
    }
}
