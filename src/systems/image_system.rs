use conrod_core::image::{Id, Map};
use glium::Display;
use glium::texture::{RawImage2d, SrgbTexture2d};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::support::{ExtensionIs, ErrToString};
use super::{EventSystem, AppEvent, events as e};
use crate::support::LogError;

#[derive(Debug, Copy, Clone)]
struct FrameData {
    pub id: Id,
    pub w: u32,
    pub h: u32,
    delay: Duration,
}

pub struct ImageSystem<'a> {
    image_map: Map<SrgbTexture2d>,
    frames: Vec<FrameData>,
    current_frame: usize,
    last_update: Instant,
    display: &'a Display,
}

impl<'a> ImageSystem<'a> {
    pub fn new(display: &'a Display) -> Self {
        ImageSystem {
            image_map: conrod_core::image::Map::new(),
            frames: Vec::new(),
            current_frame: 0,
            last_update: Instant::now(),
            display,
        }
    }

    pub fn get_map(&self) -> &Map<SrgbTexture2d> { &self.image_map }

    pub fn time_to_next_update(&self) -> Option<Duration> {
        if self.frames.len() < 2 { return None; }

        let frame = &self.frames[self.current_frame];
        let now = Instant::now();

        let diff = now.duration_since(self.last_update);

        frame.delay.checked_sub(diff)
            .or_else(|| Some(Duration::new(0, 0)))
    }

    pub fn update(&mut self, events: &mut EventSystem) -> Result<(), String> {
        let new_events: Vec<_> = events.events()
            .filter_map(|event| match event {
                AppEvent::Load(file) => self.load_file(&file),
                _ => None,
            }).collect();
        for event in new_events {
            events.push(event);
        }

        if self.frames.len() > 1 {
            let now = Instant::now();

            let diff = now.duration_since(self.last_update);

            let frame = &self.frames[self.current_frame];
            if frame.delay <= diff {
                self.current_frame = (self.current_frame + 1) % self.frames.len();
                self.last_update = now;
                let cur = &self.frames[self.current_frame];
                events.push(e::Image::SwapImageId(cur.id).into());
                log::trace!("update image frame: {}, {:?} <= {:?}", self.current_frame, frame.delay, diff);
            }
        }

        Ok(())
    }
}

// Image loading
impl<'a> ImageSystem<'a> {
    fn unload_image(&mut self) {
        for frame in &self.frames {
            log::debug!("Unloading image: {:?}", frame.id);
            self.image_map.remove(frame.id);
        }

        self.frames = Vec::new();
        self.current_frame = 0;
    }

    fn load_file(&mut self, path: &Path) -> Option<AppEvent> {
        log::info!("Loading image from path: {}", path.display());
        self.unload_image();

        if !path.exists() || !path.is_file() {
            log::error!("Could not load image from path which is not a file: {}", path.display());
            return None;
        }

        let images = ImageLoader::load(path).log_err()?;

        log::debug!("Converting raw frames to textures");
        let mut frames = Vec::new();
        for (raw, delay) in images {
            let w = raw.width;
            let h = raw.height;
            if let Some(texture) = SrgbTexture2d::new(self.display, raw).log_err() {
                frames.push(FrameData {
                    id: self.image_map.insert(texture),
                    w,
                    h,
                    delay,
                });
            }
        }

        self.frames = frames;
        self.current_frame = 0;
        self.last_update = Instant::now();

        if let Some(frame) = self.frames.get(0) {
            Some(e::Image::Loaded {
                id: frame.id,
                w: frame.w,
                h: frame.h,
            }.into())
        } else {
            None
        }
    }

    pub fn load_resource_image(&mut self, buffer: &[u8]) -> Result<Id, String> {
        let image = image::load_from_memory(buffer).err_to_string()?;
        let (texture, _) = texture_from_image(self.display, image)?;
        Ok(self.image_map.insert(texture))
    }
}

fn texture_from_image(display: &Display, image: image::DynamicImage) -> Result<(SrgbTexture2d, (u32, u32)), String> {
    let rgba = image.to_rgba();
    let dimensions = rgba.dimensions();
    let raw = RawImage2d::from_raw_rgba_reversed(&rgba.into_raw(), dimensions);
    SrgbTexture2d::new(display, raw)
        .map(|t| (t, dimensions))
        .err_to_string()
}

struct ImageLoader;

impl ImageLoader {
    pub fn load(path: &Path) -> Result<Vec<(RawImage2d<u8>, Duration)>, String> {
        if !path.exists() || !path.is_file() {
            return Err(format!("Could not load image from path which is not a file: {}", path.display()));
        }

        if path.extension_is("gif") {
            ImageLoader::load_gif(path)
        } else {
            ImageLoader::load_static_image(path)
        }
    }

    fn load_gif(path: &Path) -> Result<Vec<(RawImage2d<u8>, Duration)>, String> {
        use gif::Decoder;
        use gif_dispose::{Screen, RGBA8};
        use std::fs::File;

        let file = File::open(path).err_to_string()?;
        let decoder = Decoder::new(file);
        let mut reader = decoder.read_info().err_to_string()?;
        let mut screen: Screen<RGBA8> = Screen::from_reader(&reader);
        let mut frames = Vec::new();
        let w = screen.pixels.width() as u32;
        let h = screen.pixels.height() as u32;

        while let Some(frame) = reader.read_next_frame().err_to_string()? {
            let frame: &gif::Frame = frame;
            screen.blit_frame(frame).err_to_string()?;

            if frame.delay == 0 {
                log::info!("Frame delay is zero, blitting next frame immediately");
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

            frames.push((raw, Duration::from_millis(frame.delay as u64 * 10)));
        }

        if frames.is_empty() {
            Err("Image contained no frames".to_owned())
        } else {
            Ok(frames)
        }
    }

    fn load_static_image(path: &Path) -> Result<Vec<(RawImage2d<u8>, Duration)>, String> {
        let image = image::open(path).err_to_string()?;
        let rgba = image.to_rgba();
        let dimensions = rgba.dimensions();
        let raw = RawImage2d::from_raw_rgba_reversed(&rgba.into_raw(), dimensions);

        Ok(vec![(raw, Duration::default())])
    }
}