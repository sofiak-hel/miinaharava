//! Contains relevant structures to display and control the visual element of
//! the minesweeper.
//!
//! Contains [GameWindow] for the representation of the actual window element,
//! and also [Game] for drawing on said Window and receiving events.

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

/// Represents the Window for the Game, which is then used by Game to render
/// stuff on.
pub struct GameWindow {
    pub(crate) canvas: Canvas<Window>,
    pub(crate) texture_creator: TextureCreator<WindowContext>,
    pub(crate) event_pump: EventPump,
}

impl GameWindow {
    /// Opens the window for rendering, must be called at the start of
    /// everything.
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

/// Represents the visual game itself. Does not contain Minefield itself, but is
/// able to render it and show relevant user-information, requires Window to work.
///
/// See also [Minefield] for the actual mechanical game part.
pub struct Game<'a> {
    minefield_renderer: MinefieldRenderer<'a>,
    fonts: [Font; 1],
    layout: Layout<Color>,
    canvas: &'a mut Canvas<Window>,
    font_texture: FontTexture<'a>,
    event_pump: &'a mut EventPump,
    last: Instant,
    quit: bool,
    /// Shown timer, publicly available so it can be formatted or edited at
    /// will.
    pub timer: f32,
    /// Whether the timer should be paused. If true, updated at [Game::update]
    pub timer_paused: bool,
    /// Extra layout for use in the implemented binary. Meant for use for text
    /// which helps with user input.
    pub extra_layout: Layout<Color>,
    pub extra_layout_size: f32,
    pub extra_layout_keybind_color: Color,
    pub extra_layout_color: Color,
}

/// Events propagated from sdl2 [EventPump], contains [Event]s themselves and
/// also current mouse position.
pub struct GameEvents {
    /// Array of events, keypresses, window quits etc.
    pub events: Vec<Event>,
    /// Current mouse position in pixels.
    pub mouse_pos: (i32, i32),
    /// Timedelta (in seconds) since last frame
    pub delta: f32,
}

impl<'a> Game<'a> {
    /// Initializes renderer with [GameWindow]
    pub fn init(window: &'a mut GameWindow) -> Game<'a> {
        let minefield_area = Rect::new(0, 0, 900, 720);
        let minefield_renderer = MinefieldRenderer::init(&window.texture_creator, minefield_area);

        let roboto_regular = Font::from_bytes(FONT, Default::default()).unwrap();
        let font_texture = FontTexture::new(&window.texture_creator).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            x: 910.,
            y: 30.,
            max_width: Some(370.),
            horizontal_align: HorizontalAlign::Center,
            ..Default::default()
        });
        let mut extra_layout = Layout::new(CoordinateSystem::PositiveYDown);
        extra_layout.reset(&LayoutSettings {
            x: 910.,
            y: 230.,
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
            extra_layout_color: Color::RGB(0xFF, 0xFF, 0),
            extra_layout_keybind_color: Color::RED,
            extra_layout_size: 32.,
        }
    }

    /// Mechanical update, does not draw onto the Window, but instead returns
    /// current event data and processes timer.
    pub fn update(&mut self) -> Option<GameEvents> {
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
                } => self.exit(),
                _ => {}
            }
        }
        if self.quit {
            None
        } else {
            let mouse_state = self.event_pump.mouse_state();
            Some(GameEvents {
                events,
                mouse_pos: (mouse_state.x(), mouse_state.y()),
                delta,
            })
        }
    }

    /// Simply draws onto the [GameWindow] canvas according to current data,
    /// game state and which tile from said minefield should show as hovered, if
    /// any.
    pub fn draw<const W: usize, const H: usize>(
        &mut self,
        minefield: &Minefield<W, H>,
        hover_tile: Option<Coord<W, H>>,
    ) {
        self.canvas.set_draw_color(Color::RGB(40, 40, 40));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(64, 64, 150));
        self.canvas
            .fill_rect(self.minefield_renderer.get_target::<W, H>())
            .unwrap();
        self.minefield_renderer
            .draw(minefield, self.canvas, hover_tile);

        self.layout.clear();
        self.append_text(format!("{}, {}\n", W, H), None, None);
        self.append_text(format!("{} mines\n", minefield.mines), None, None);
        self.append_text(format!("{:.1}\n", self.timer), None, None);
        match minefield.game_state() {
            GameState::GameOver => {
                self.append_text("Game over!", None, Some(Color::RGB(0xFF, 0, 0)));
            }
            GameState::Victory => self.append_text("Victory!", None, Some(Color::RGB(0, 0xFF, 0))),
            _ => {}
        }

        let _ = self
            .font_texture
            .draw_text(self.canvas, &self.fonts, self.layout.glyphs());
        let _ = self
            .font_texture
            .draw_text(self.canvas, &self.fonts, self.extra_layout.glyphs());

        self.canvas.present();
    }

    /// Attempt to convert screen-pixel-coordinates into game-tile-coordinates.
    pub fn get_coord<const W: usize, const H: usize>(
        &self,
        mouse: (i32, i32),
    ) -> Option<Coord<W, H>> {
        self.minefield_renderer.get_coord(mouse)
    }

    /// Forcibly exists the game, update will stop returning things.
    pub fn exit(&mut self) {
        self.quit = true;
    }

    /// Append text into [Game::extra_layout].
    pub fn append_extra<T: Into<String>>(
        &mut self,
        text: T,
        size: Option<f32>,
        color: Option<Color>,
    ) {
        let size = size.unwrap_or(self.extra_layout_size);
        let color = color.unwrap_or(self.extra_layout_color);
        self.extra_layout.append(
            &self.fonts,
            &TextStyle::with_user_data(&text.into(), size, 0, color),
        );
    }

    /// Same as [Game::append_extra], but instead appends a very specific type
    /// of text and coloring, in the following format:
    ///
    /// `<red>[keybind]<clear> description`
    pub fn append_keybind<T: Into<String>, U: Into<String>>(&mut self, keybind: T, description: U) {
        self.append_extra(
            format!("[{}] ", keybind.into()),
            None,
            Some(self.extra_layout_keybind_color),
        );
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
