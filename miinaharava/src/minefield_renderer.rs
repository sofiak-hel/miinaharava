use sdl2::{
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    rwops::RWops,
    surface::Surface,
    video::{Window, WindowContext},
};

use crate::minefield::{Cell, Coord, Minefield};

static ATLAS_BYTES: &[u8] = include_bytes!("./resources/atlas.bmp");
const SIZE: u32 = 16;
const STRIDE: i32 = 16;

pub struct MinefieldRenderer<'a> {
    atlas: Texture<'a>,
    target: Rect,
}

impl<'a> MinefieldRenderer<'a> {
    pub fn init(texture_creator: &'a TextureCreator<WindowContext>, target: Rect) -> Self {
        let surface = Surface::load_bmp_rw(&mut RWops::from_bytes(ATLAS_BYTES).unwrap()).unwrap();
        MinefieldRenderer {
            atlas: texture_creator
                .create_texture_from_surface(surface)
                .unwrap(),
            target,
        }
    }

    pub fn draw<const W: usize, const H: usize>(
        &self,
        minefield: &Minefield<W, H>,
        canvas: &mut Canvas<Window>,
        mouse: Option<(i32, i32)>,
    ) {
        let (pos_x, pos_y, total_w, total_h) = self.get_target::<W, H>().into();
        let (w, h) = (total_w / W as u32, total_h / H as u32);
        for y in 0..H {
            for x in 0..W {
                let dest_rect = Rect::new(
                    pos_x + (x as u32 * w) as i32,
                    pos_y + (y as u32 * h) as i32,
                    w,
                    h,
                );
                let hover = mouse.map(|m| dest_rect.contains_point(m)).unwrap_or(false);
                let source_rect = Rect::from(source(minefield.field[y][x], hover));
                canvas.copy(&self.atlas, source_rect, dest_rect).unwrap();
            }
        }
    }

    pub fn get_coord<const W: usize, const H: usize>(
        &self,
        mouse: (i32, i32),
    ) -> Option<Coord<W, H>> {
        let (pos_x, pos_y, total_w, total_h) = self.get_target::<W, H>().into();
        let (w, h) = (total_w as i32 / W as i32, total_h as i32 / H as i32);
        let x = (mouse.0 - pos_x) / w;
        let y = (mouse.1 - pos_y) / h;
        if x >= 0 && x < W as i32 && y >= 0 && y < H as i32 {
            Some(Coord(x as usize, y as usize))
        } else {
            None
        }
    }

    pub fn get_target<const W: usize, const H: usize>(&self) -> Rect {
        let w = self.target.width() as usize / W;
        let h = self.target.height() as usize / H;
        let scale = w.min(h);
        Rect::from_center(self.target.center(), (W * scale) as u32, (H * scale) as u32)
    }
}

const fn source(cell: Cell, hover: bool) -> (i32, i32, u32, u32) {
    let pos = match cell {
        Cell::Hidden if hover => (0, 3),
        Cell::Flag if hover => (1, 3),

        Cell::Empty => (0, 0),
        Cell::Hidden => (1, 0),
        Cell::Flag => (2, 0),
        Cell::Mine => (3, 0),
        Cell::Label(x) => (((x - 1) % 4) as i32, ((x - 1) / 4 + 1) as i32),
    };
    (pos.0 * STRIDE, pos.1 * STRIDE, SIZE, SIZE)
}
