use std::convert::Into;
use crate::data::{File, FileSort, Rating};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Image(Image),
    Load(File),
    Nav(Nav),
    Sort(FileSort),
    Filter(Filter),
    SetMeta(SetMeta),
}

#[derive(Debug, Clone)]
pub enum Image {
    Loaded {
        id: conrod_core::image::Id,
        w: u32,
        h: u32,
    },
    SwapImageId(conrod_core::image::Id),
}

impl Into<AppEvent> for Image {
    fn into(self) -> AppEvent { AppEvent::Image(self) }
}

#[derive(Debug, Clone)]
pub enum Nav {
    ImageNext,
    ImagePrev,
    ImageIndex(usize),
}

impl Into<AppEvent> for Nav {
    fn into(self) -> AppEvent { AppEvent::Nav(self) }
}

#[derive(Debug, Clone)]
pub enum Filter {
    Text(String),
    Rating(Option<Rating>),
}

impl Into<AppEvent> for Filter {
    fn into(self) -> AppEvent { AppEvent::Filter(self) }
}

#[derive(Debug, Clone)]
pub enum SetMeta {
    Rating(Option<Rating>),
}

impl Into<AppEvent> for SetMeta {
    fn into(self) -> AppEvent { AppEvent::SetMeta(self) }
}
