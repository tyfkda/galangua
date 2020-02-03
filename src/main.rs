extern crate sdl2;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::thread;
use std::time::{Duration, SystemTime};

const PAD_L: u8 = 1 << 0;
const PAD_R: u8 = 1 << 1;
const PAD_A: u8 = 1 << 2;

fn get_key_bit(key: Keycode) -> u8 {
    return match key {
        Keycode::Left => PAD_L,
        Keycode::Right => PAD_R,
        Keycode::Space => PAD_A,
        _ => 0,
    }
}

struct FpsCalc {
    fps: i32,
    last_fps_time: SystemTime,
    ndraw: i32,
}

impl FpsCalc {
    fn new() -> FpsCalc {
        FpsCalc {
            fps: 0,
            last_fps_time: SystemTime::now(),
            ndraw: 0,
        }
    }

    fn update(&mut self) -> bool {
        self.ndraw += 1;
        let now = SystemTime::now();
        if now.duration_since(self.last_fps_time).expect("Time went backwards").as_secs() < 1 {
            return false;
        }

        self.fps = self.ndraw;
        self.ndraw = 0;
        self.last_fps_time = now;
        //self.window.set_title(&format!("FPS {}", self.fps));
        true
    }
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Garaga", 240 * 2, 320 * 2)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    //let texture = texture_creator.load_texture(png).unwrap();

    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, 16 * 2, 16 * 2)
        .map_err(|e| e.to_string())?;
    // Create a red-green gradient
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..16*2 {
            for x in 0..16*2 {
                let offset = y*pitch + x*3;
                buffer[offset] = 255;
                buffer[offset + 1] = 255;
                buffer[offset + 2] = 255;
            }
        }
    })?;

    let mut fps_calc = FpsCalc::new();
    let frame_duration = Duration::from_micros(1_000_000 / 60);
    let mut next_update_time = SystemTime::now() + frame_duration;

    let mut x = 100 * 2;
    let y = 280 * 2;
    let mut pad: u8 = 0;
    let mut mx = 0;
    let mut my = -1;

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                | Event::KeyDown { keycode: Some(key), .. } => {
                    pad = pad | get_key_bit(key);
                },
                | Event::KeyUp { keycode: Some(key), .. } => {
                    pad = pad & !get_key_bit(key);
                },
                _ => {},
            }
        }

        if pad & PAD_L != 0 {
            x = x - 2 * 2;
            if x < 8 * 2 {
                x = 8 * 2;
            }
        }
        if pad & PAD_R != 0 {
            x = x + 2 * 2;
            if x > (240 - 8) * 2 {
                x = (240 - 8) * 2;
            }
        }
        if pad & PAD_A != 0 && my < 0 {
            mx = x;
            my = y;
        }
        if my >= 0 {
            my -= 8 * 2;
        }

        canvas.clear();
        canvas.copy(&texture, None, Some(Rect::new(x - 8 * 2, y - 8 * 2, 16 * 2, 16 * 2)))?;
        if my >= 0 {
            canvas.copy(&texture, None, Some(Rect::new(mx - 1 * 2, my - 3 * 2, 2 * 2, 6 * 2)))?;
        }
        canvas.present();

        if fps_calc.update() {
            canvas.window_mut().set_title(&format!("FPS {}", fps_calc.fps)).expect("");
        }

        let now = SystemTime::now();
        if now < next_update_time {
            let d = next_update_time.duration_since(now).expect("");
            thread::sleep(d);
        }
        next_update_time += frame_duration;
    }

    Ok(())
}
