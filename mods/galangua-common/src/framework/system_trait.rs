pub trait SystemTrait {
    fn get_u32(&self, key: &str) -> Option<u32>;
    fn set_u32(&mut self, key: &str, value: u32);

    fn is_touch_device(&self) -> bool;

    fn play_se(&mut self, channel: u32, filename: &str);
}
