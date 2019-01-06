use crate::data::FileSort;

#[derive(Copy, Clone, Debug)]
pub enum Action {
    ImageNext,
    ImagePrev,
    Select(usize),
    Sort(FileSort),
}