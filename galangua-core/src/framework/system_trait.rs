pub trait SystemTrait {
    fn get_u32(&self, key: &str) -> Option<u32>;
    fn set_u32(&mut self, key: &str, value: u32);
}
