mod conrod;
mod path;

pub use self::conrod::*;
pub use self::path::*;

pub trait ErrToString<T> {
    fn err_to_string(self) -> Result<T, String>;
}

impl<T, E> ErrToString<T> for Result<T, E>
    where E: std::fmt::Display {
    fn err_to_string(self) -> Result<T, String> {
        self.map_err(|e| format!("{}", e))
    }
}
