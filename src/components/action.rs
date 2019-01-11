use std::path::PathBuf;

use crate::data::{FileSort, Rating};

#[derive(Clone, Debug)]
pub enum Action {
    ImageNext,
    ImagePrev,
    Select(usize),
    Sort(FileSort),
    LoadImage(PathBuf),
    FilterByText(String),
    SetRating(Option<Rating>),
}