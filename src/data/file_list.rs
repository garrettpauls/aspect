use std::path::{Path, PathBuf};
use std::fmt;
use std::convert::AsRef;
use std::cmp::Ordering;

use super::file::File;

#[derive(Debug)]
pub struct FileList {
    pub files: Vec<File>,
    current_index: usize,
    current_sort: FileSort,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FileSort {
    Name,
    LastModified,
}

pub static FILE_SORT_METHODS: &[FileSort] = &[FileSort::Name, FileSort::LastModified];

impl FileSort {
    pub fn compare(&self, a: &File, b: &File) -> Ordering {
        match self {
            FileSort::Name => a.path.file_name().cmp(&b.path.file_name()),
            FileSort::LastModified => a.last_modified().cmp(&b.last_modified()),
        }
    }
}

impl fmt::Display for FileSort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for FileSort {
    fn as_ref(&self) -> &str {
        match self {
            FileSort::Name => "Name",
            FileSort::LastModified => "Last Modified",
        }
    }
}

impl FileList {
    pub fn from_environment() -> Option<Self> {
        use std::env::args;

        let paths: Vec<_> = args().map(|a| PathBuf::from(a)).filter(|p| p.exists()).collect();
        let files = paths.iter().filter_map(|p| FileList::from_file(p)).next()
            .or(paths.iter().filter_map(|p| FileList::from_dir(p)).next());

        files
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        if !is_image_file(path) {
            return None;
        }

        let dir = path.parent()?;
        let mut list = FileList::from_dir(dir)?;

        let (i, _) = list.files.iter().enumerate().find(|(_, f)| f.path == path)?;
        list.current_index = i;

        Some(list)
    }

    pub fn from_dir(path: &Path) -> Option<Self> {
        if !path.exists() || !path.is_dir() {
            return None;
        }

        let mut file_names = Vec::new();
        for entry in path.read_dir().ok()? {
            let entry = entry.ok()?;
            if !is_image_file(&entry.path()) {
                continue;
            }

            file_names.push(File {
                path: entry.path()
            });
        }

        let mut list = FileList {
            files: file_names,
            current_index: 0,
            current_sort: FileSort::LastModified,
        };

        list.sort_by(FileSort::Name);

        Some(list)
    }

    pub fn current_index(&self) -> usize { self.current_index }

    pub fn current_sort(&self) -> &FileSort { &self.current_sort }

    pub fn set_current(&mut self, current: usize) {
        self.current_index = current % self.files.len();
    }

    pub fn increment_current(&mut self, amount: i64) {
        let i = (self.current_index as i64 + amount) % self.files.len() as i64;
        let i = if i < 0 { self.files.len() as i64 + i } else { i } as usize;
        self.current_index = i;
    }

    pub fn get_file(&self, index: usize) -> Option<&File> {
        self.files.get(index)
    }

    pub fn sort_by(&mut self, property: FileSort) {
        if self.current_sort == property {
            log::info!("Sort files by {} skipped due to already sorted", property);
            return;
        }

        log::info!("Sort files by {}", property);

        self.current_sort = property;
        let selected = self
            .get_file(self.current_index)
            .map(|f| f.path.clone());

        { self.files.sort_by(|a, b| property.compare(a, b)); }

        let new_idx = if let Some(selected) = selected {
            self.files.iter().enumerate()
                .find(|(_, i)| i.path == selected)
                .map(|(i, _)| i)
                .unwrap_or(0)
        } else {
            0
        };

        log::info!("Restoring index to {} after sort", new_idx);
        self.set_current(new_idx);
    }
}

pub static SUPPORTED_FILE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp"];

fn is_image_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let ext = path.extension().map(|x| x.to_str()).unwrap_or(None).unwrap_or("").to_lowercase();
    SUPPORTED_FILE_EXTENSIONS.contains(&&ext[..])
}