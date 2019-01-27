mod conrod;
mod path;

pub use self::conrod::*;
pub use self::path::*;

pub trait ErrToString<T> {
    fn err_to_string(self) -> Result<T, String>;
}

impl<T, E> ErrToString<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn err_to_string(self) -> Result<T, String> {
        self.map_err(|e| format!("{}", e))
    }
}

pub trait LogError<T> {
    fn log_err(self) -> Option<T>;
}

impl<T, E> LogError<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn log_err(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                log::error!("{}", e);
                None
            }
        }
    }
}

pub trait ToNone {
    fn none<O>(&self) -> Option<O> {
        None
    }
}

impl ToNone for () {}
