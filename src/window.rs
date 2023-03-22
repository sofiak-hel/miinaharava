use sdl2::{
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
    EventPump,
};

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
