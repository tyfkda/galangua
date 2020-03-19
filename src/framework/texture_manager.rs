use sdl2::image::LoadTexture;
use sdl2::render::{Texture, WindowCanvas};

use crate::util::resource_manager::ResourceManager;

pub struct TextureManager {
    resource_manager: ResourceManager<Texture>,
}

impl TextureManager {
    pub fn new() -> TextureManager {
        TextureManager {
            resource_manager: ResourceManager::new(),
        }
    }

    pub fn load(&mut self, canvas: &mut WindowCanvas, base_path: &str, filenames: &Vec<&str>) -> Result<(), String> {
        self.resource_manager.load(base_path, filenames, |path: &str| {
            let texture_creator = canvas.texture_creator();
            texture_creator.load_texture(path)
        })
    }

    pub fn get(&self, key: &str) -> Option<&Texture> {
        self.resource_manager.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Texture> {
        self.resource_manager.get_mut(key)
    }
}
