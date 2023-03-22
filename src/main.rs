//! hello

#![deny(clippy::all)]
#![allow(missing_docs)]

use fontdue::layout::{CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle};
use fontdue::Font;
use fontdue_sdl2::FontTexture;
use minefield::Minefield;
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::future::Pending;
use std::time::Instant;

use crate::minefield::GameState;

pub mod minefield;
pub mod renderer;

static FONT: &[u8] = include_bytes!("./Outfit-Medium.ttf");

/// main docs
pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("minesweeper", 1280, 720)
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

    let renderer = Renderer::init(&texture_creator);

    let roboto_regular = Font::from_bytes(FONT, Default::default()).unwrap();
    let fonts = &[roboto_regular];
    let mut font_texture = FontTexture::new(&texture_creator).unwrap();
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: 900.,
        y: 100.,
        max_width: Some(380.),
        horizontal_align: HorizontalAlign::Center,
        ..Default::default()
    });

    const size: (usize, usize) = (10, 10);
    let mut minefield = Minefield::<{ size.0 }, { size.1 }>::generate(10);

    let minefield_area = Rect::new(0, 0, 900, 720);
    let w = minefield_area.width() as usize / size.0;
    let h = minefield_area.height() as usize / size.1;
    let scale = w.min(h);
    let minefield_target = Rect::from_center(
        minefield_area.center(),
        (size.0 * scale) as u32,
        (size.1 * scale) as u32,
    );

    let mut time = 0.;

    'running: loop {
        let now = Instant::now();
        let delta = (Instant::now() - last).as_secs_f32();
        last = now;

        if minefield.game_state() == GameState::Pending {
            time += delta;
        }

        canvas.set_draw_color(Color::RGB(40, 40, 40));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(64, 64, 150));
        canvas.fill_rect(minefield_area).unwrap();

        for event in event_pump.poll_iter() {
            match event {
                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => {
                    if minefield.game_state() == GameState::Pending {
                        match mouse_btn {
                            MouseButton::Left => {
                                if let Some(coord) = Renderer::get_coord(minefield_target, (x, y)) {
                                    minefield.reveal(coord).unwrap();
                                }
                            }
                            MouseButton::Right => {
                                if let Some(coord) = Renderer::get_coord(minefield_target, (x, y)) {
                                    minefield.flag(coord).unwrap();
                                }
                            }
                            _ => {}
                        }
                    }
                }

                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        let mouse_state = event_pump.mouse_state();
        let drag_pos = if (mouse_state.left() || mouse_state.right())
            && minefield.game_state() == GameState::Pending
        {
            Some((mouse_state.x(), mouse_state.y()))
        } else {
            None
        };

        renderer.draw(&minefield, &mut canvas, minefield_target, drag_pos);

        layout.clear();
        layout.append(
            fonts,
            &TextStyle::with_user_data(
                &format!("{}, {}\n", size.0, size.1), // The text to lay out
                32.0,                                 // The font size
                0,                                    // The font index (Roboto Regular)
                Color::RGB(0xFF, 0xFF, 0),            // The color of the text
            ),
        );
        layout.append(
            fonts,
            &TextStyle::with_user_data(
                &format!("{time:.1}\n"),   // The text to lay out
                32.0,                      // The font size
                0,                         // The font index (Roboto Regular)
                Color::RGB(0xFF, 0xFF, 0), // The color of the text
            ),
        );
        let text_style = match minefield.game_state() {
            GameState::GameOver => Some(TextStyle::with_user_data(
                "Game over!",
                32.0,
                0,
                Color::RGB(0xFF, 0, 0),
            )),
            GameState::Victory => Some(TextStyle::with_user_data(
                "Victory!",
                32.0,
                0,
                Color::RGB(0, 0xFF, 0),
            )),
            GameState::Pending => None,
        };
        if let Some(text_style) = text_style {
            layout.append(fonts, &text_style);
        }
        let _ = font_texture.draw_text(&mut canvas, fonts, layout.glyphs());

        canvas.present();
    }
}
