use std::path::PathBuf;

use crate::data::FileSort;

#[derive(Clone, Debug)]
pub enum Action {
    ImageNext,
    ImagePrev,
    Select(usize),
    Sort(FileSort),
    LoadImage(PathBuf),
    FilterByText(String),
}