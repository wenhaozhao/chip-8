use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::Sdl;
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

const SCALE: u32 = 16;
const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

pub struct Display {
    frame_buffer: Vec<u8>,
    renderer: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
}

impl Display {
    pub fn new(sdl_ctx: &Sdl) -> Self {
        let video_subsystem = sdl_ctx.video().unwrap();
        let window = video_subsystem.window("chip-8", WIDTH * SCALE, HEIGHT * SCALE)
            .position_centered()
            .build().unwrap();
        let mut renderer = window.into_canvas().accelerated().build().unwrap();
        let texture_creator = renderer.texture_creator();
        renderer.set_scale(SCALE as f32, SCALE as f32).unwrap();
        renderer.clear();
        Display {
            frame_buffer:  vec![0; WIDTH as usize * HEIGHT as usize * 4],
            renderer,
            texture_creator,
        }
    }

    pub fn clear(&mut self) {
        self.frame_buffer = vec![0; self.frame_buffer.len()];
        self.renderer.set_draw_color(Color::BLACK);
        self.renderer.clear();
        self.renderer.present();
    }

    pub fn set_pixel(&mut self, x: usize, y: usize) -> bool {
        let x = if x >= 64 { x % 64 } else { x };
        let y = if y >= 32 { y % 32 } else { y };
        let position = (y * 64 + x) * 4; // Since Each pixel occupy 4 byte in vec
        // pixel is already set
        if (self.frame_buffer[position]
            | self.frame_buffer[position + 1]
            | self.frame_buffer[position + 2]
            | self.frame_buffer[position + 3])
            != 0
        {
            // unset pixel
            self.frame_buffer[position] = 0; // unset A (alpha)
            self.frame_buffer[position + 1] = 0; // unset R
            self.frame_buffer[position + 2] = 0; // unset G
            self.frame_buffer[position + 3] = 0; // unset B
            true
        } else {
            // else set pixel
            self.frame_buffer[position] = 254; //set  A (alpha)
            self.frame_buffer[position + 1] = 100; // set R
            self.frame_buffer[position + 2] = 254; // set G
            self.frame_buffer[position + 3] = 100; // set B
            false
        }
    }

    pub fn render(&mut self) {
        self.renderer.clear();
        let surface = Surface::from_data(
            self.frame_buffer.as_mut(),
            WIDTH, HEIGHT,
            64 * 4,
            PixelFormatEnum::ARGB8888,
        ).unwrap();
        let texture = self.texture_creator
            .create_texture_from_surface(surface)
            .unwrap();
        self.renderer.copy(&texture, None, None).unwrap();
        self.renderer.present();
    }
}