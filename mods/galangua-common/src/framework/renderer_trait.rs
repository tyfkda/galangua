use crate::framework::types::Vec2I;

pub trait RendererTrait {
    fn load_textures(&mut self, base_path: &str, filenames: &[&str]);
    fn load_sprite_sheet(&mut self, filename: &str);
    fn clear(&mut self);
    fn set_texture_color_mod(&mut self, tex_name: &str, r: u8, g: u8, b: u8);
    fn set_sprite_texture_color_mod(&mut self, sprite_name: &str, r: u8, g: u8, b: u8);
    fn draw_str(&mut self, tex_name: &str, x: i32, y: i32, text: &str);
    fn draw_sprite(&mut self, sprite_name: &str, pos: &Vec2I);
    fn draw_sprite_rot(&mut self, sprite_name: &str, pos: &Vec2I, angle: u8,
                       center: Option<&Vec2I>);
    fn set_draw_color(&mut self, r: u8, g: u8, b: u8);
    fn fill_rect(&mut self, dst: Option<[&Vec2I; 2]>);
}
