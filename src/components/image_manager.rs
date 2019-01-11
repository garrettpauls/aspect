use conrod_core::image::{Id, Map};
use glium::Display;
use glium::texture::{RawImage2d, SrgbTexture2d};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::support::ExtensionIs;

#[derive(Debug, Clone)]
pub struct ImageData {
    pub id: Id,
    pub w: u32,
    pub h: u32,
    hash: u64,
    delay: Duration,
}

impl PartialEq for ImageData {
    fn eq(&self, other: &ImageData) -> bool { self.hash == other.hash }
}

pub struct ImageManager<'a> {
    image_map: Map<SrgbTexture2d>,
    frames: Vec<ImageData>,
    current_frame: usize,
    last_update: Instant,
    display: &'a Display,
}

impl<'a> ImageManager<'a> {
    pub fn new(display: &'a Display) -> ImageManager {
        ImageManager {
            image_map: conrod_core::image::Map::new(),
            frames: Vec::new(),
            current_frame: 0,
            last_update: Instant::now(),
            display,
        }
    }

    pub fn get_map(&self) -> &Map<SrgbTexture2d> { &self.image_map }

    pub fn current(&self) -> Option<&ImageData> { self.frames.get(self.current_frame) }

    pub fn time_to_next_update(&self) -> Option<Duration> {
        if self.frames.len() < 2 { return None; }

        let frame = &self.frames[self.current_frame];
        let now = Instant::now();

        let diff = now.duration_since(self.last_update);

        frame.delay.checked_sub(diff)
            .or_else(|| Some(Duration::new(0, 0)))
    }

    fn unload_image(&mut self) {
        for frame in &self.frames {
            self.image_map.remove(frame.id);
        }

        self.frames = Vec::new();
        self.current_frame = 0;
    }

    pub fn update(&mut self) {
        if self.frames.len() < 2 { return; }

        let now = Instant::now();

        let diff = now.duration_since(self.last_update);

        let frame = &self.frames[self.current_frame];
        if frame.delay <= diff {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_update = now;
            log::info!("update image frame: {}, {:?} <= {:?}", self.current_frame, frame.delay, diff);
        }
    }

    pub fn load_image(&mut self, path: &Path) -> Result<(), String> {
        log::info!("Loading image from path: {}", path.display());
        self.unload_image();

        if !path.exists() || !path.is_file() {
            return Err("not a file".to_owned());
        }

        let result = if path.extension_is("gif") {
            self.load_gif(path)
        } else {
            self.load_static_image(path)
        };

        self.last_update = Instant::now();
        result
    }

    fn load_gif(&mut self, path: &Path) -> Result<(), String> {
        use gif::Decoder;
        use gif_dispose::{Screen, RGBA8};
        use std::fs::File;

        let file = File::open(path).map_err(|e| format!("{}", e))?;
        let decoder = Decoder::new(file);
        let mut reader = decoder.read_info().map_err(|e| format!("{}", e))?;
        let mut screen: Screen<RGBA8> = Screen::from_reader(&reader);
        let mut frames = Vec::new();
        let hash = hash_path(path);
        let w = screen.pixels.width() as u32;
        let h = screen.pixels.height() as u32;

        while let Some(frame) = reader.read_next_frame().map_err(|e| format!("{}", e))? {
            let frame: &gif::Frame = frame;
            screen.blit_frame(frame).map_err(|e| format!("{}", e))?;

            if frame.delay == 0 {
                log::warn!("Frame delay is zero, blitting next frame immediately");
                continue;
            }

            let mut buf = Vec::new();
            for p in &screen.pixels.buf {
                buf.push(p.r);
                buf.push(p.g);
                buf.push(p.b);
                buf.push(p.a);
            }

            let raw = RawImage2d::from_raw_rgba_reversed(&buf, (w, h));
            let texture = SrgbTexture2d::new(self.display, raw).map_err(|e| format!("{}", e))?;

            frames.push(ImageData {
                id: self.image_map.insert(texture),
                w,
                h,
                delay: Duration::from_millis(frame.delay as u64 * 10),
                hash,
            });
        }

        self.frames = frames;
        self.current_frame = 0;
        Ok(())
    }

    fn load_static_image(&mut self, path: &Path) -> Result<(), String> {
        let (image, (w, h)) = load_image_from_file(self.display, path)?;

        self.current_frame = 0;
        self.frames = vec![ImageData {
            id: self.image_map.insert(image),
            w,
            h,
            hash: hash_path(&path),
            delay: Duration::default(),
        }];

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

fn hash_path(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}