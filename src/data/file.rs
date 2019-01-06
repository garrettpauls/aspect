use std::path::PathBuf;
use std::time::SystemTime;
use std::fmt;

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
}

impl File {
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