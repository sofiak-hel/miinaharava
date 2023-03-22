use std::time::Instant;

use fontdue::{
    layout::{CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle},
    Font,
};
use fontdue_sdl2::FontTexture;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
    EventPump,
};

use crate::{
    minefield::{Coord, GameState, Minefield},
    minefield_renderer::MinefieldRenderer,
};

static FONT: &[u8] = include_bytes!("./resources/Outfit-Medium.ttf");

pub struct GameWindow {
    pub(crate) canvas: Canvas<Window>,
    pub(crate) texture_creator: TextureCreator<WindowContext>,
    pub(crate) event_pump: EventPump,
}

impl GameWindow {
    pub fn start() -> Self {
        let sdl_context = sdl2::init().unwrap();

        let event_pump = sdl_context.event_pump().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("minesweeper", 1280, 720)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().present_vsync().build().unwrap();
        let texture_creator = canvas.texture_creator();

        GameWindow {
            canvas,
            texture_creator,
            event_pump,
        }
    }
}

pub struct Game<'a> {
    minefield_renderer: MinefieldRenderer<'a>,
    fonts: [Font; 1],
    layout: Layout<Color>,
    canvas: &'a mut Canvas<Window>,
    font_texture: FontTexture<'a>,
    event_pump: &'a mut EventPump,
    last: Instant,
    quit: bool,
    pub timer: f32,
    pub timer_paused: bool,
    pub extra_layout: Layout<Color>,
}

impl<'a> Game<'a> {
    pub fn init(window: &'a mut GameWindow) -> Game<'a> {
        let minefield_area = Rect::new(0, 0, 900, 720);
        let minefield_renderer = MinefieldRenderer::init(&window.texture_creator, minefield_area);

        let roboto_regular = Font::from_bytes(FONT, Default::default()).unwrap();
        let font_texture = FontTexture::new(&window.texture_creator).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            x: 910.,
            y: 100.,
            max_width: Some(370.),
            horizontal_align: HorizontalAlign::Center,
            ..Default::default()
        });
        let mut extra_layout = Layout::new(CoordinateSystem::PositiveYDown);
        extra_layout.reset(&LayoutSettings {
            x: 910.,
            y: 250.,
            max_width: Some(370.),
            horizontal_align: HorizontalAlign::Left,
            ..Default::default()
        });

        Game {
            minefield_renderer,
            layout,
            canvas: &mut window.canvas,
            fonts: [roboto_regular],
            font_texture,
            event_pump: &mut window.event_pump,
            timer: 0.,
            last: Instant::now(),
            timer_paused: true,
            quit: false,
            extra_layout,
        }
    }

    pub fn update(&mut self) -> Option<Vec<Event>> {
        let now = Instant::now();
        let delta = (Instant::now() - self.last).as_secs_f32();
        self.last = now;
        if !self.timer_paused {
            self.timer += delta;
        }
        let events: Vec<Event> = self.event_pump.poll_iter().collect();
        for event in &events {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.quit = true,
                _ => {}
            }
        }
        if self.quit {
            None
        } else {
            Some(events)
        }
    }

    pub fn draw<const W: usize, const H: usize>(
        &mut self,
        minefield: &Minefield<W, H>,
        show_hover: bool,
    ) {
        self.canvas.set_draw_color(Color::RGB(40, 40, 40));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(64, 64, 150));
        self.canvas
            .fill_rect(self.minefield_renderer.get_target::<W, H>())
            .unwrap();

        let mouse_pos = if show_hover {
            let state = self.event_pump.mouse_state();
            Some((state.x(), state.y()))
        } else {
            None
        };
        self.minefield_renderer
            .draw(minefield, self.canvas, mouse_pos);

        self.layout.clear();
        self.append_text(format!("{}, {}\n", W, H), None, None);
        self.append_text(format!("{} mines\n", minefield.mines), None, None);
        self.append_text(format!("{:.1}\n", self.timer), None, None);
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
            self.layout.append(&self.fonts, &text_style);
        }

        let _ = self
            .font_texture
            .draw_text(self.canvas, &self.fonts, self.layout.glyphs());
        let _ = self
            .font_texture
            .draw_text(self.canvas, &self.fonts, self.extra_layout.glyphs());

        self.canvas.present();
    }

    pub fn get_coord<const W: usize, const H: usize>(
        &self,
        mouse: (i32, i32),
    ) -> Option<Coord<W, H>> {
        self.minefield_renderer.get_coord(mouse)
    }

    pub fn exit(&mut self) {
        self.quit = true;
    }

    pub fn append_extra<T: Into<String>>(
        &mut self,
        text: T,
        size: Option<f32>,
        color: Option<Color>,
    ) {
        let size = size.unwrap_or(32.);
        let color = color.unwrap_or(Color::RGB(0xFF, 0xFF, 0));
        self.extra_layout.append(
            &self.fonts,
            &TextStyle::with_user_data(&text.into(), size, 0, color),
        );
    }

    pub fn append_keybind<T: Into<String>, U: Into<String>>(&mut self, keybind: T, description: U) {
        self.append_extra(format!("[{}] ", keybind.into()), None, Some(Color::RED));
        self.append_extra(format!("{}\n", description.into()), None, None);
    }

    fn append_text<T: Into<String>>(&mut self, text: T, size: Option<f32>, color: Option<Color>) {
        let size = size.unwrap_or(32.);
        let color = color.unwrap_or(Color::RGB(0xFF, 0xFF, 0));
        self.layout.append(
            &self.fonts,
            &TextStyle::with_user_data(&text.into(), size, 0, color),
        );
    }
}
