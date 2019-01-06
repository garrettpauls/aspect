use conrod_core::image::{Id, Map};
use glium::texture::SrgbTexture2d;
use std::path::Path;

pub struct ImageManager {
    image_map: Map<SrgbTexture2d>,
    image_id: Option<Id>,
}

impl ImageManager {
    pub fn new() -> ImageManager {
        ImageManager {
            image_map: conrod_core::image::Map::new(),
            image_id: None,
        }
    }

    pub fn get_map(&self) -> &Map<SrgbTexture2d> { &self.image_map }

    pub fn get_image_id(&self) -> &Option<Id> { &self.image_id }

    pub fn load_image(&mut self, path: &Path) -> Result<(), String> {
        log::info!("Loading image from path: {}", path.display());
        Err("Not implemented".to_owned())
    }
}