use conrod_core::theme::Theme;
use conrod_core::color;

pub fn default_theme() -> Theme {
    let mut theme = Theme::default();
    theme.background_color = color::DARK_CHARCOAL;
    theme.label_color = color::LIGHT_GREY;
    theme
}