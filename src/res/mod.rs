use conrod_core::image::Id;

use crate::components::ImageManager;

pub struct ImageIds {
    pub star_outline: Id,
    pub star_filled: Id,
}

pub struct Resources {
    pub images: ImageIds,
}

impl Resources {
    pub fn load(image_manager: &mut ImageManager) -> Result<Self, String> {
        let images = ImageIds {
            star_filled: image_manager.load_resource_image(include_bytes!("images/baseline_star_white_48dp.png"))?,
            star_outline: image_manager.load_resource_image(include_bytes!("images/baseline_star_border_white_48dp.png"))?,
        };

        Ok(Resources {
            images
        })
    }
}
