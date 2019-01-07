use conrod_core::image::{Id, Map};
use glium::Display;
use glium::texture::{RawImage2d, SrgbTexture2d};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ImageData {
    pub id: Id,
    pub w: u32,
    pub h: u32,
}

pub struct ImageManager {
    image_map: Map<SrgbTexture2d>,
    current: Option<ImageData>,
}

impl ImageManager {
    pub fn new() -> ImageManager {
        ImageManager {
            image_map: conrod_core::image::Map::new(),
            current: None,
        }
    }

    pub fn get_map(&self) -> &Map<SrgbTexture2d> { &self.image_map }

    pub fn current(&self) -> &Option<ImageData> { &self.current }

    fn unload_image(&mut self) {
        if let Some(data) = &self.current {
            self.image_map.remove(data.id);
        }

        self.current = None;
    }

    pub fn load_image(&mut self, display: &Display, path: &Path) -> Result<(), String> {
        log::info!("Loading image from path: {}", path.display());

        if !path.exists() || !path.is_file() {
            self.unload_image();
            return Err("not a file".to_owned());
        }

        let (image, (w, h)) = match load_image_from_file(display, path) {
            Ok(img) => img,
            Err(e) => {
                self.unload_image();
                return Err(e);
            }
        };

        if let Some(ImageData { id, .. }) = self.current {
            self.image_map.replace(id, image);
        } else {
            self.current = Some(ImageData {
                id: self.image_map.insert(image),
                w,
                h,
            });
        }

        Ok(())
    }
}

fn load_image_from_file(display: &Display, path: &Path) -> Result<(SrgbTexture2d, (u32, u32)), String> {
    let (raw_image, dimensions) = image::open(path)
        .map(|i| {
            let rgba = i.to_rgba();
            let dimensions = rgba.dimensions();
            (RawImage2d::from_raw_rgba_reversed(&rgba.into_raw(), dimensions), dimensions)
        })
        .map_err(|e| format!("{}", e))?;

    SrgbTexture2d::new(display, raw_image)
        .map(|t| (t, dimensions))
        .map_err(|e| format!("{}", e))
}