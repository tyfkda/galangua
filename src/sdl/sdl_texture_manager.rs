use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::{WindowContext};

use super::resource_manager::ResourceManager;

pub struct SdlTextureManager<'a> {
    resource_manager: ResourceManager<(Texture<'a>, TextureCreator<WindowContext>)>,
}

impl<'a> SdlTextureManager<'a> {
    pub fn new() -> Self {
        Self {
            resource_manager: ResourceManager::new(),
        }
    }

    pub fn load(&mut self, canvas: &mut WindowCanvas, base_path: &str,
                filenames: &[&str]) -> Result<(), String> {
        self.resource_manager.load(base_path, filenames, |path: &str| {
            let creator = canvas.texture_creator();
            creator.load_texture(path)
                .map(|texture| (texture, creator))
        })
    }

    pub fn get(&self, key: &str) -> Option<&Texture<'a>> {
        self.resource_manager.get(key).map(|pair| &pair.0)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Texture<'a>> {
        self.resource_manager.get_mut(key).map(|pair| &mut pair.0)
    }
}
