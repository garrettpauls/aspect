use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FileList {
    files: Vec<File>,
    current_index: usize,
}

#[derive(Debug)]
pub struct File {
    path: PathBuf
}

impl FileList {
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
}

fn is_image_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let ext = path.extension().map(|x| x.to_str()).unwrap_or(None).unwrap_or("").to_lowercase();
    ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "bmp"
}