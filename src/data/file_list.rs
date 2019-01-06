use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug)]
pub struct FileList {
    pub files: Vec<File>,
    current_index: usize,
}

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
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

        Some(FileList {
            files: file_names,
            current_index: 0,
        })
    }

    pub fn current_index(&self) -> usize { self.current_index }

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
}

impl File {
    pub fn size(&self) -> u64 {
        self.path.metadata().map(|m| m.len()).unwrap_or(0)
    }

    pub fn last_modified(&self) -> SystemTime {
        self.path.metadata()
            .map(|m| m.modified().unwrap_or(SystemTime::UNIX_EPOCH))
            .unwrap_or(SystemTime::UNIX_EPOCH)
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