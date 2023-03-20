use sdl2::event::Event;
use sdl2::image::InitFlag;
use sdl2::joystick::Joystick;
use sdl2::keyboard::Keycode;
use sdl2::mixer::{AUDIO_S16LSB, DEFAULT_CHANNELS};
use sdl2::Sdl;
use std::thread;
use std::time::{Duration, SystemTime};

use galangua_common::framework::{AppTrait, VKey};

use crate::sdl::sdl_renderer::SdlRenderer;

type MapKeyFunc = fn(Keycode) -> Option<VKey>;

const FPS: u32 = 60;
const MIN_FPS: u32 = 15;

pub struct SdlAppFramework<App: AppTrait<SdlRenderer>> {
    sdl_context: Sdl,
    last_update_time: SystemTime,

    app: App,
    map_key: MapKeyFunc,

    #[cfg(debug_assertions)]
    fast_forward: bool,
}

impl<App: AppTrait<SdlRenderer>> SdlAppFramework<App> {
    pub fn new(app: App, map_key: MapKeyFunc) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;

        Ok(Self {
            sdl_context,
            last_update_time: SystemTime::now(),
            app,
            map_key,

            #[cfg(debug_assertions)]
            fast_forward: false,
        })
    }

    pub fn run(&mut self, title: &str, width: u32, height: u32, scale: u32, fullscreen: bool) -> Result<(), String> {
        let video_subsystem = self.sdl_context.video()?;
        let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

        let _joystick = self.set_up_joystick()?;

        let mut window_builder = video_subsystem
            .window(title, width * scale, height * scale);
        if fullscreen {
            window_builder.fullscreen();
        } else {
            window_builder
                .position_centered()
                .resizable();
        }
        let window = window_builder
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;
        if fullscreen {
            self.sdl_context.mouse().show_cursor(false);
        }

        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        let _audio = self.sdl_context.audio()?;

        let frequency = 44_100;
        let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
        let channels = DEFAULT_CHANNELS; // Stereo
        let chunk_size = 1_024;
        sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
        let _mixer_context = sdl2::mixer::init(
            sdl2::mixer::InitFlag::MP3 | sdl2::mixer::InitFlag::FLAC | sdl2::mixer::InitFlag::MOD | sdl2::mixer::InitFlag::OGG
        )?;

        // Number of mixing channels available for sound effect `Chunk`s to play
        // simultaneously.
        sdl2::mixer::allocate_channels(4);

        let mut renderer = SdlRenderer::new(canvas, (width, height));

        self.app.init(&mut renderer);

        self.last_update_time = SystemTime::now();
        let mut skip_count = 0;
        'running: loop {
            if !self.pump_events()? {
                break 'running;
            }

            #[cfg(debug_assertions)]
            let step = if self.fast_forward { 10 } else { 1 + skip_count };
            #[cfg(not(debug_assertions))]
            let step = 1 + skip_count;

            for _ in 0..step {
                if !self.app.update() {
                    break 'running;
                }
            }
            self.app.draw(&mut renderer);
            renderer.present();

            skip_count = self.wait_frame(Duration::from_micros(1_000_000 / FPS as u64));
        }
        Ok(())
    }

    pub fn pump_events(&mut self) -> Result<bool, String> {
        let mut event_pump = self.sdl_context.event_pump()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return Ok(false);
                }
                Event::KeyDown { keycode: Some(key), .. } => {
                    #[cfg(debug_assertions)]
                    if key == Keycode::LShift {
                        self.fast_forward = true;
                    }
                    if let Some(vkey) = (self.map_key)(key) {
                        self.app.on_key(vkey, true);
                    }
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    #[cfg(debug_assertions)]
                    if key == Keycode::LShift {
                        self.fast_forward = false;
                    }
                    if let Some(vkey) = (self.map_key)(key) {
                        self.app.on_key(vkey, false);
                    }
                }
                Event::JoyAxisMotion { axis_idx, value, .. } => {
                    let dir = if value > 10_000 { 1 } else if value < -10_000 { -1 } else { 0 };
                    self.app.on_joystick_axis(axis_idx, dir);
                }
                Event::JoyButtonDown { button_idx, .. } => {
                    self.app.on_joystick_button(button_idx, true);
                }
                Event::JoyButtonUp { button_idx, .. } => {
                    self.app.on_joystick_button(button_idx, false);
                }
                _ => {}
            }
        }
        Ok(true)
    }

    pub fn wait_frame(&mut self, duration: Duration) -> u32 {
        let next_update_time = self.last_update_time + duration;
        let now = SystemTime::now();
        if now < next_update_time {
            let wait = next_update_time.duration_since(now).expect("");
            thread::sleep(wait);
            self.last_update_time = next_update_time;
            0
        } else {
            let late = now.duration_since(next_update_time).expect("");
            let skip_count = (late.as_millis() as f32 / duration.as_millis() as f32).floor() as u32;
            if skip_count <= FPS / MIN_FPS {
                self.last_update_time = next_update_time + duration * skip_count;
                skip_count
            } else {
                self.last_update_time = now;
                FPS / MIN_FPS
            }
        }
    }

    fn set_up_joystick(&mut self) -> Result<Option<Joystick>, String> {
        let joystick_subsystem = self.sdl_context.joystick()?;
        let available = joystick_subsystem
            .num_joysticks()
            .map_err(|e| format!("can't enumerate joysticks: {e}"))?;
        let joystick = (0..available).find_map(|id| match joystick_subsystem.open(id) {
            Ok(c) => Some(c),
            Err(_e) => None,
        });
        Ok(joystick)
    }
}
