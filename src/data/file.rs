use std::fmt;
use std::path::PathBuf;
use std::time::SystemTime;
use std::convert::{Into, From};

#[derive(Debug, Clone)]
pub struct Rating(usize);

impl Rating {
    pub fn as_i64(&self) -> i64 { self.0 as i64 }
}

impl From<usize> for Rating {
    fn from(rating: usize) -> Self { Rating::from(rating as i64) }
}

impl From<i64> for Rating {
    fn from(rating: i64) -> Self {
        Rating(rating.max(1).min(5) as usize)
    }
}

impl Into<usize> for Rating {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<i64> for Rating {
    fn into(self) -> i64 { self.0 as i64 }
}

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub rating: Option<Rating>,
}

impl From<&str> for File {
    fn from(path: &str) -> Self {
        File {
            path: PathBuf::from(path),
            rating: None,
        }
    }
}

impl File {
    pub fn name(&self) -> String {
        match self.path.file_name() {
            Some(s) => s.to_string_lossy().to_string(),
            None => "".to_owned()
        }
    }

    pub fn size(&self) -> FileSize {
        FileSize(self.path.metadata().map(|m| m.len()).unwrap_or(0))
    }

    pub fn last_modified(&self) -> SystemTime {
        self.path.metadata()
            .map(|m| m.modified().unwrap_or(SystemTime::UNIX_EPOCH))
            .unwrap_or(SystemTime::UNIX_EPOCH)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FileSize(u64);

#[derive(Debug, PartialEq)]
pub enum FileSizeUnit {
    Byte(u64),
    KiloByte(f64),
    MegaByte(f64),
    GigaByte(f64),
}

impl FileSize {
    pub fn reduce(&self) -> FileSizeUnit {
        if self.0 < 1024 {
            return FileSizeUnit::Byte(self.0);
        }

        let mut remaining = self.0 as f64 / 1024.0;
        if remaining < 1024.0 {
            return FileSizeUnit::KiloByte(remaining);
        }

        remaining /= 1024.0;
        if remaining < 1024.0 {
            return FileSizeUnit::MegaByte(remaining);
        }

        remaining /= 1024.0;
        return FileSizeUnit::GigaByte(remaining);
    }
}

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.reduce().fmt(f)
    }
}

impl fmt::Display for FileSizeUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileSizeUnit::Byte(b) => write!(f, "{} B", b),
            FileSizeUnit::KiloByte(kb) => write!(f, "{:.2} KB", kb),
            FileSizeUnit::MegaByte(mb) => write!(f, "{:.2} MB", mb),
            FileSizeUnit::GigaByte(gb) => write!(f, "{:.2} GB", gb),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce() {
        assert_eq!(FileSize(500).reduce(), FileSizeUnit::Byte(500));
        assert_eq!(FileSize(1024).reduce(), FileSizeUnit::KiloByte(1.0));
        assert_eq!(FileSize(2048).reduce(), FileSizeUnit::KiloByte(2.0));
        assert_eq!(FileSize(1048576).reduce(), FileSizeUnit::MegaByte(1.0));
        assert_eq!(FileSize(1073741824).reduce(), FileSizeUnit::GigaByte(1.0));
        assert_eq!(FileSize(1099511627776).reduce(), FileSizeUnit::GigaByte(1024.0));
    }
}