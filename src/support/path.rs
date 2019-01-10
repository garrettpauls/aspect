use std::path::{Path, PathBuf};

pub trait ExtensionIs {
    fn extension_is(&self, ext: &str) -> bool;
}

impl ExtensionIs for Path {
    fn extension_is(&self, ext: &str) -> bool {
        self.extension().map(|e| e.to_string_lossy().to_lowercase() == ext.to_lowercase()).unwrap_or(false)
    }
}

impl ExtensionIs for PathBuf {
    fn extension_is(&self, ext: &str) -> bool {
        self.as_path().extension_is(ext)
    }
}
