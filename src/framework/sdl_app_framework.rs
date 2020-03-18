use sdl2::Sdl;
use sdl2::event::Event;
use sdl2::image::InitFlag;
use sdl2::keyboard::Keycode;
use std::thread;
use std::time::{Duration, SystemTime};

use super::AppTrait;
use super::sdl_renderer::SdlRenderer;

pub struct SdlAppFramework<App: AppTrait<SdlRenderer>> {
    sdl_context: Sdl,
    last_update_time: SystemTime,

    app: Box<App>,
}

impl<App: AppTrait<SdlRenderer>> SdlAppFramework<App> {
    pub fn new(app: Box<App>) -> Result<SdlAppFramework<App>, String> {
        let sdl_context = sdl2::init()?;

        Ok(SdlAppFramework {
            sdl_context,
            last_update_time: SystemTime::now(),
            app,
        })
    }

    pub fn run(&mut self, title: &str, width: u32, height: u32) -> Result<(), String> {
        let video_subsystem = self.sdl_context.video()?;
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
        let mut renderer = SdlRenderer::new(canvas);

        self.app.init(&mut renderer)?;

        'running: loop {
            if !self.pump_events()? {
                break 'running;
            }

            self.app.update();
            self.app.draw(&mut renderer)?;

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
