//use std::time::{Duration, SystemTime};

pub struct FpsCalc {
    fps: i32,
    //last_fps_time: SystemTime,
    ndraw: i32,
}

impl FpsCalc {
    pub fn new() -> Self {
        Self {
            fps: 0,
            //last_fps_time: SystemTime::now(),
            ndraw: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        /*self.ndraw += 1;
        let now = SystemTime::now();
        if now.duration_since(self.last_fps_time).expect("Time went backwards").as_secs() < 1 {
            return false;
        }

        self.fps = self.ndraw;
        self.ndraw = 0;
        self.last_fps_time = self.last_fps_time + Duration::from_secs(1);*/
        true
    }

    pub fn fps(&self) -> i32 {
        self.fps
    }
}
