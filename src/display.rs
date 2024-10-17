extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

const WIDTH_PIXEL_COUNT: u8 = 64;
const HEIGHT_PIXEL_COUNT: u8 = 32;

pub struct DisplayChip8 {
    pixel_size: u32,
    pixels: [bool; (WIDTH_PIXEL_COUNT as usize) * (HEIGHT_PIXEL_COUNT as usize)],
    pub canvas: WindowCanvas,
}

impl DisplayChip8 {
    pub fn new(pixel_size: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                "CHIP8",
                pixel_size * (WIDTH_PIXEL_COUNT as u32),
                pixel_size * (HEIGHT_PIXEL_COUNT as u32),
            )
            .position_centered()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();
        Self {
            pixel_size,
            pixels: [false; (WIDTH_PIXEL_COUNT as usize) * (HEIGHT_PIXEL_COUNT as usize)],
            canvas,
        }
    }

    fn render(&mut self) -> Result<(), String> {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.canvas.set_draw_color(Color::WHITE);
        let mut pixels_drawn: Vec<Rect> = Vec::new();
        for (position, pixel) in self.pixels.iter().enumerate() {
            if *pixel {
                let x = position as u32 % (WIDTH_PIXEL_COUNT as u32);
                let y = position as u32 / (WIDTH_PIXEL_COUNT as u32);
                pixels_drawn.push(Rect::new(
                    (x * self.pixel_size) as i32,
                    (y * self.pixel_size) as i32,
                    self.pixel_size,
                    self.pixel_size,
                ));
            }
        }
        self.canvas.fill_rects(&pixels_drawn)
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.pixels.iter_mut().for_each(|e| *e = false);
    }

    pub fn draw(&mut self, x_position: u8, y_position: u8, bytes: &[u8]) -> bool {
        let x_position = x_position & (WIDTH_PIXEL_COUNT - 1) as u8;
        let y_position = y_position & (HEIGHT_PIXEL_COUNT - 1) as u8;
        let mut did_turn_off_pixel = false;
        for (byte_number, byte) in bytes.iter().enumerate() {
            let y_position = y_position + byte_number as u8;
            let mut mask = 0b10000000;
            for bit in 0..8 {
                let x_position = x_position + bit;
                let is_flipped = ((byte & mask) >> (8 - bit - 1)) == 1;
                if is_flipped {
                    did_turn_off_pixel =
                        self.flip_pixel(x_position, y_position) || did_turn_off_pixel;
                }
                mask = mask.rotate_right(1);
            }
        }
        let _ = self.render();
        did_turn_off_pixel
    }

    pub fn show(&mut self) {
        self.canvas.present();
    }

    fn flip_pixel(&mut self, x_position: u8, y_position: u8) -> bool {
        if x_position < WIDTH_PIXEL_COUNT && y_position < HEIGHT_PIXEL_COUNT {
            let position =
                (x_position as usize) + (y_position as usize) * (WIDTH_PIXEL_COUNT as usize);
            self.pixels[position] = !self.pixels[position];
            !self.pixels[position]
        } else {
            false
        }
    }
}
