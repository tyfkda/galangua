extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

pub struct MyShot {
    x: i32,
    y: i32,
}

impl MyShot {
    pub fn new(x: i32, y: i32) -> MyShot {
        MyShot {
            x,
            y,
        }
    }

    pub fn update(&mut self) -> bool {
        self.y -= 8;

        self.y >= 0
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &Texture) -> Result<(), String> {
        canvas.copy(&texture, None, Some(Rect::new((self.x - 1) * 2, (self.y - 3) * 2, 2 * 2, 6 * 2)))?;

        Ok(())
    }
}
