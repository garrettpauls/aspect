use conrod_core::image::{Id, Map};
use glium::texture::{RawImage2d, SrgbTexture2d};
use glium::Display;
use std::path::Path;
use std::time::{Duration, Instant};

use super::{events as e, AppEvent, EventSystem};
use crate::data::File;
use crate::support::{ErrToString, ExtensionIs};

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

    pub fn get_map(&self) -> &Map<SrgbTexture2d> {
        &self.image_map
    }

    pub fn time_to_next_update(&self) -> Option<Duration> {
        if self.frames.len() < 2 {
            return None;
        }

        let frame = &self.frames[self.current_frame];
        let now = Instant::now();

        let diff = now.duration_since(self.last_update);

        frame
            .delay
            .checked_sub(diff)
            .or_else(|| Some(Duration::new(0, 0)))
    }

    pub fn update(&mut self, events: &mut EventSystem) -> Result<(), String> {
        let new_events: Vec<_> = events
            .events()
            .filter_map(|event| match event {
                AppEvent::Load(file) => self.load_file(&file),
                _ => None,
            })
            .collect();
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
                log::trace!(
                    "update image frame: {}, {:?} <= {:?}",
                    self.current_frame,
                    frame.delay,
                    diff
                );
            }
        }

        Ok(())
    }
}

// Image loading
impl<'a> ImageSystem<'a> {
    fn unload_image(&mut self) {
        for frame in &self.frames {
            self.image_map.remove(frame.id);
        }

        self.frames = Vec::new();
        self.current_frame = 0;
    }

    fn load_file(&mut self, file: &File) -> Option<AppEvent> {
        let path = &file.path;
        log::info!("Loading image from path: {}", path.display());
        self.unload_image();

        if !path.exists() || !path.is_file() {
            log::error!(
                "Could not load image from path which is not a file: {}",
                path.display()
            );
            return None;
        }

        let result = if path.extension_is("gif") {
            self.load_gif(path)
        } else {
            self.load_static_image(path)
        };

        self.last_update = Instant::now();
        if let Ok(frame) = result {
            Some(
                e::Image::Loaded {
                    id: frame.id,
                    w: frame.w,
                    h: frame.h,
                    file: file.clone(),
                }
                .into(),
            )
        } else {
            None
        }
    }

    fn load_gif(&mut self, path: &Path) -> Result<FrameData, String> {
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
            let texture = SrgbTexture2d::new(self.display, raw).err_to_string()?;

            frames.push(FrameData {
                id: self.image_map.insert(texture),
                w,
                h,
                delay: Duration::from_millis(frame.delay as u64 * 10),
            });
        }

        self.frames = frames;
        self.current_frame = 0;

        if let Some(cur) = self.frames.get(self.current_frame) {
            Ok(*cur)
        } else {
            Err("Image contained not frames".to_owned())
        }
    }

    fn load_static_image(&mut self, path: &Path) -> Result<FrameData, String> {
        let (image, (w, h)) = load_image_from_file(self.display, path)?;

        let frame = FrameData {
            id: self.image_map.insert(image),
            w,
            h,
            delay: Duration::default(),
        };
        self.current_frame = 0;
        self.frames = vec![frame];

        Ok(frame)
    }

    pub fn load_resource_image(&mut self, buffer: &[u8]) -> Result<Id, String> {
        let image = image::load_from_memory(buffer).err_to_string()?;
        let (texture, _) = texture_from_image(self.display, image)?;
        Ok(self.image_map.insert(texture))
    }
}

fn load_image_from_file(
    display: &Display,
    path: &Path,
) -> Result<(SrgbTexture2d, (u32, u32)), String> {
    let image = image::open(path).err_to_string()?;
    texture_from_image(display, image)
}

fn texture_from_image(
    display: &Display,
    image: image::DynamicImage,
) -> Result<(SrgbTexture2d, (u32, u32)), String> {
    let rgba = image.to_rgba();
    let dimensions = rgba.dimensions();
    let raw = RawImage2d::from_raw_rgba_reversed(&rgba.into_raw(), dimensions);
    SrgbTexture2d::new(display, raw)
        .map(|t| (t, dimensions))
        .err_to_string()
}
