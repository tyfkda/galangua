use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

pub fn draw_str(canvas: &mut WindowCanvas, texture: &Texture, x: i32, y: i32, text: &str) -> Result<(), String> {
    let w = 16;
    let h = 16;
    let mut x = x;

    for c in text.chars() {
        let u: i32 = ((c as i32) - (' ' as i32)) % 16 * 8;
        let v: i32 = ((c as i32) - (' ' as i32)) / 16 * 8;
        canvas.copy(&texture,
                    Some(Rect::new(u, v, 8, 8)),
                    Some(Rect::new(x, y, w, h)))?;
        x += w as i32;
    }

    Ok(())
}
