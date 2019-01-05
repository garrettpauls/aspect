use conrod_core::color;
use conrod_core::position::{Padding, Range};
use conrod_core::theme::Theme;

pub fn default_theme() -> Theme {
    let mut theme = Theme::default();
    theme.background_color = color::DARK_CHARCOAL;
    theme.label_color = color::LIGHT_GREY;
    theme.shape_color = color::DARK_BLUE;
    theme.padding = Padding { x: Range::new(8.0, 8.0), y: Range::new(8.0, 8.0) };
    theme
}