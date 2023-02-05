use sdl2::mixer::{Chunk, MAX_VOLUME};

pub struct SdlAudio {
    channels: Vec<Option<Chunk>>,
    base_volume: i32,
}

impl SdlAudio {
    pub fn new(channel_count: u32, base_volume: f32) -> Self {
        let mut channels = Vec::with_capacity(channel_count as usize);
        channels.resize_with(channel_count as usize, || None);

        Self {
            channels,
            base_volume: (MAX_VOLUME as f32 * base_volume) as i32,
        }
    }

    pub fn play_se(&mut self, channel: u32, filename: &str) {
        if channel < self.channels.len() as u32 {
            let path = format!("{}.ogg", filename);
            match Chunk::from_file(path) {
                Err(err) => println!("{}: {}", err, filename),
                Ok(mut chunk) => {
                    chunk.set_volume(self.base_volume);
                    sdl2::mixer::Channel::all().play(&chunk, 0)
                        .expect("Play music failed");
                    self.channels[channel as usize] = Some(chunk);
                }
            }
        }
    }
}
