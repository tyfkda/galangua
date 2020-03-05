use sdl2::Sdl;
use sdl2::event::Event;
use sdl2::image::InitFlag;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use std::thread;
use std::time::{Duration, SystemTime};

use super::App;

pub struct SdlAppFramework {
    pub sdl_context: Sdl,
    pub canvas: WindowCanvas,
    pub last_update_time: SystemTime,

    pub app: Box<dyn App>,
}

impl SdlAppFramework {
    pub fn new(title: &str, width: u32, height: u32, app: Box<dyn App>) -> Result<SdlAppFramework, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

        let window = video_subsystem
            .window(title, width, height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;
        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        Ok(SdlAppFramework {
            sdl_context,
            canvas,
            last_update_time: SystemTime::now(),
            app,
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.app.init(&mut self.canvas)?;

        'running: loop {
            if !self.pump_events()? {
                break 'running;
            }

            self.app.update();
            self.app.draw(&mut self.canvas)?;

            self.wait_frame(Duration::from_micros(1_000_000 / 60));
        }
        Ok(())
    }

    pub fn pump_events(&mut self) -> Result<bool, String> {
        let mut event_pump = self.sdl_context.event_pump()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return Ok(false);
                },
                | Event::KeyDown { keycode: Some(key), .. } => {
                    self.app.on_key_down(key);
                },
                | Event::KeyUp { keycode: Some(key), .. } => {
                    self.app.on_key_up(key);
                }
                _ => {}
            }
        }
        Ok(true)
    }

    pub fn wait_frame(&mut self, duration: Duration) {
        let next_update_time = self.last_update_time + duration;
        let now = SystemTime::now();
        if now < next_update_time {
            let d = next_update_time.duration_since(now).expect("");
            thread::sleep(d);
            self.last_update_time += duration;
        } else {
            self.last_update_time = now;
        }
    }
}
