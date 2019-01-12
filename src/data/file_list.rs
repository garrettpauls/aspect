use std::convert::AsRef;
use std::path::{Path, PathBuf};
use std::fmt;
use std::ops::Drop;

use super::{File, Filter, Rating};
use super::persist::PersistenceManager;
use crate::support::{ExtensionIs, LogError};

#[derive(Debug)]
pub struct FileList {
    files: Vec<File>,
    current_index: usize,
    current_sort: FileSort,
    filter: Filter,
    filtered_files: Vec<File>,
    persist: Option<PersistenceManager>,
}

impl Drop for FileList {
    fn drop(&mut self) {
        use std::mem::replace;
        if let Some(p) = replace(&mut self.persist, None) {
            if let Err(e) = p.close() {
                log::error!("Failed to close persistence manager: {}", e);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FileSort {
    Name,
    LastModified,
    Random,
}

pub static FILE_SORT_METHODS: &[FileSort] = &[FileSort::Name, FileSort::LastModified, FileSort::Random];

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
            FileSort::Random => "Random",
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
                path: entry.path(),
                rating: None,
            });
        }

        let persist = PersistenceManager::open_dir(path)
            .map_err(|e| log::error!("Could not initialize persistence manager, functionality is limited! {}", e))
            .ok();

        Some(FileList::from_files(file_names, persist))
    }

    pub fn from_files(files: Vec<File>, persist: Option<PersistenceManager>) -> Self {
        let mut list = FileList {
            files,
            current_index: 0,
            current_sort: FileSort::Name,
            filter: Filter::default(),
            filtered_files: Vec::new(),
            persist,
        };

        list.apply_sort();

        list
    }
}

impl FileList {
    pub fn current(&self) -> Option<&File> { self.files.get(self.current_index) }

    fn current_mut(&mut self) -> Option<&mut File> { self.files.get_mut(self.current_index) }

    pub fn current_index(&self) -> usize { self.current_index }

    pub fn current_sort(&self) -> &FileSort { &self.current_sort }

    pub fn len(&self) -> usize { self.files.len() }

    pub fn set_current(&mut self, current: usize) {
        self.current_index = current % self.files.len();
    }

    pub fn set_rating(&mut self, rating: Option<Rating>) {
        if let Some(current) = self.current_mut() {
            current.rating = rating;
        }

        match (&self.current(), &self.persist) {
            (Some(current), Some(persist)) => persist.set_rating(current, &current.rating),
            _ => Ok(())
        }.log_err();
    }

    pub fn increment_current(&mut self, amount: i64) {
        let i = (self.current_index as i64 + amount) % self.files.len() as i64;
        let i = if i < 0 { self.files.len() as i64 + i } else { i } as usize;
        self.current_index = i;
    }

    pub fn get_file(&self, index: usize) -> Option<&File> {
        self.files.get(index)
    }

    pub fn get_filter(&self) -> &Filter { &self.filter }

    pub fn sort_by(&mut self, property: FileSort) {
        if self.current_sort == property {
            log::info!("Sort files by {} skipped due to already sorted", property);
            return;
        }

        self.current_sort = property;
        self.apply_sort();
    }

    fn apply_sort(&mut self) {
        log::info!("Sort files by {}", self.current_sort);

        let selected = self
            .get_file(self.current_index)
            .map(|f| f.path.clone());

        {
            use rand::{thread_rng, seq::SliceRandom};
            match self.current_sort {
                FileSort::Name => self.files.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name())),
                FileSort::LastModified => self.files.sort_by(|a, b| a.last_modified().cmp(&b.last_modified())),
                FileSort::Random => self.files.shuffle(&mut thread_rng()),
            }
        }

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

    pub fn apply_filter(&mut self, filter: Filter) {
        log::info!("Filtering files: {:?}", filter);

        let is_subset = filter.is_subset_of(&self.filter);
        if !is_subset {
            log::info!("Resetting file list for filtering");
            while let Some(f) = self.filtered_files.pop() {
                self.files.push(f);
            }
        }

        let mut i = 0;
        while i < self.files.len() {
            let file = &self.files[i];
            if !filter.matches(file) {
                log::info!("Filtering out file: {}", file.path.display());
                let f = self.files.remove(i);
                self.filtered_files.push(f);
                if self.current_index > 0 && self.current_index > i {
                    self.current_index -= 1;
                }
            } else {
                i += 1;
            }
        }

        if self.current_index >= self.len() {
            self.current_index = self.len() - 1;
        }

        self.filter = filter;
        if !is_subset {
            self.apply_sort();
        }
    }
}

pub static SUPPORTED_FILE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "gif"];

fn is_image_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    SUPPORTED_FILE_EXTENSIONS.iter().any(|ext| path.extension_is(*ext))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    pub fn filter_syncs_selected_item() {
        const AC3: &str = r"C:\files\3ac.png";
        let files = vec![
            r"C:\files\1a.png",
            r"C:\files\2b.png",
            AC3,
            r"C:\files\4bc.png",
            r"C:\files\5ac.png",
        ].iter().map(|f| File { path: PathBuf::from(*f), rating: None }).collect();
        let mut list = FileList::from_files(files, None);

        list.set_current(2);
        list.apply_filter(Filter::default().with_name("a"));
        println!("{:#?}", list);
        assert_eq!(list.current().unwrap().path.to_string_lossy(), AC3, "filter by 'a'");

        list.apply_filter(Filter::default().with_name("ac"));
        assert_eq!(list.current().unwrap().path.to_string_lossy(), AC3, "filter by 'ac'");

        list.apply_filter(Filter::default().with_name("a"));
        assert_eq!(list.current().unwrap().path.to_string_lossy(), AC3, "filter by 'a' after 'ac'");

        list.apply_filter(Filter::default().with_name(""));
        assert_eq!(list.current().unwrap().path.to_string_lossy(), AC3, "filter by '' after 'ac'");
    }
}