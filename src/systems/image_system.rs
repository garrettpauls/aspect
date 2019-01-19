use conrod_core::image::{Id, Map};
use glium::Display;
use glium::texture::{RawImage2d, SrgbTexture2d};
use std::path::Path;
use std::time::{Duration, Instant};
use std::mem::replace;

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
    loader: Option<ImageLoader>,
}

impl<'a> ImageSystem<'a> {
    pub fn new(display: &'a Display) -> Self {
        ImageSystem {
            image_map: conrod_core::image::Map::new(),
            frames: Vec::new(),
            current_frame: 0,
            last_update: Instant::now(),
            display,
            loader: None,
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
        for event in events.events() {
            match event {
                AppEvent::Load(file) => self.load_file(&file),
                _ => (),
            }
        }

        if let Some(loader) = self.loader.take() {
            let loader = loader.update();
            match loader.try_get_image() {
                Ok(result) => {
                    if let Some(event) = self.finish_loader(result) {
                        events.push(event);
                    }
                }
                Err(loader) => self.loader = Some(loader),
            }
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

    fn load_file(&mut self, path: &Path) {
        log::info!("Loading image from path: {}", path.display());

        if !path.exists() || !path.is_file() {
            log::error!("Could not load image from path which is not a file: {}", path.display());
            self.unload_image();
            return;
        }

        if let Some(loader) = &self.loader {
            if loader.is_loading_path(&path) {
                return;
            }
        }

        let loader = ImageLoader::load(path.to_path_buf());
        if let Some(old) = replace(&mut self.loader, Some(loader)) {
            old.cancel();
        }
    }

    fn finish_loader(&mut self, result: Result<Vec<RawImageData>, String>) -> Option<AppEvent> {
        log::info!("Unloading current frames");
        self.unload_image();

        match result {
            Err(e) => {
                log::error!("Failed to load image: {}", e);
                None
            }
            Ok(images) => {
                log::debug!("Converting raw frames to textures");
                let mut frames = Vec::new();
                for raw in images {
                    let raw_image = RawImage2d::from_raw_rgba_reversed(&raw.buf, (raw.w, raw.h));
                    if let Some(texture) = SrgbTexture2d::new(self.display, raw_image).log_err() {
                        frames.push(FrameData {
                            id: self.image_map.insert(texture),
                            w: raw.w,
                            h: raw.h,
                            delay: raw.delay,
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

use std::thread;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::convert::Into;

enum ImageLoader {
    Loading(PathBuf, Arc<Mutex<bool>>, thread::JoinHandle<Result<Vec<RawImageData>, String>>),
    Completed(PathBuf, Result<Vec<RawImageData>, String>),
}

struct RawImageData {
    buf: Vec<u8>,
    w: u32,
    h: u32,
    delay: Duration,
}

impl ImageLoader {
    pub fn load(path: PathBuf) -> ImageLoader {
        if !path.exists() || !path.is_file() {
            let e = format!("Could not load image from path which is not a file: {}", path.display());
            return ImageLoader::Completed(path, Err(e));
        }

        let m = Arc::new(Mutex::new(false));
        let thread_path = path.clone();
        ImageLoader::Loading(path, m.clone(), thread::spawn(move || {
            let result = if thread_path.extension_is("gif") {
                ImageLoader::load_gif(&thread_path)
            } else {
                ImageLoader::load_static_image(&thread_path)
            };

            log::debug!("Loading image complete, notifying waiting threads");
            let mut completed = m.lock().unwrap();
            *completed = true;

            log::debug!("Loading image complete, exiting thread");
            result
        }))
    }

    pub fn is_loading_path(&self, target: &Path) -> bool {
        match self {
            ImageLoader::Completed(path, _) => path == target,
            ImageLoader::Loading(path, _, _) => path == target,
        }
    }

    pub fn update(self) -> Self {
        match self {
            ImageLoader::Loading(path, arc, handle) => {
                log::debug!("Updating loading image loader");
                if *arc.lock().unwrap() {
                    log::debug!("Loading complete, joining thread");
                    let result = handle.join().unwrap();
                    ImageLoader::Completed(path, result)
                } else {
                    log::debug!("Loading not complete");
                    ImageLoader::Loading(path, arc, handle)
                }
            }
            _ => self,
        }
    }

    pub fn try_get_image(self) -> Result<Result<Vec<RawImageData>, String>, Self> {
        match self {
            ImageLoader::Completed(_, result) => Ok(result),
            _ => Err(self),
        }
    }

    pub fn cancel(self) {
        log::info!("Cancelling load of image");
        // TODO: properly dispose of the thread
    }

    fn load_gif(path: &Path) -> Result<Vec<RawImageData>, String> {
        use gif::Decoder;
        use gif_dispose::{Screen, RGBA8};
        use std::fs::File;

        log::debug!("Loading file as gif: {}", path.display());
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

            let mut buf = Vec::with_capacity(screen.pixels.buf.len() * 4);
            for p in &screen.pixels.buf {
                buf.push(p.r);
                buf.push(p.g);
                buf.push(p.b);
                buf.push(p.a);
            }

            let raw = RawImageData {
                buf,
                w,
                h,
                delay: Duration::from_millis(frame.delay as u64 * 10),
            };

            frames.push(raw);
        }

        if frames.is_empty() {
            Err("Image contained no frames".to_owned())
        } else {
            Ok(frames)
        }
    }

    fn load_static_image(path: &Path) -> Result<Vec<RawImageData>, String> {
        log::debug!("Loading file as static image: {}", path.display());

        let image = image::open(path).err_to_string()?;
        let rgba = image.to_rgba();
        let dimensions = rgba.dimensions();
        let raw = RawImageData {
            buf: rgba.into_raw(),
            w: dimensions.0,
            h: dimensions.1,
            delay: Duration::default(),
        };

        Ok(vec![raw])
    }
}