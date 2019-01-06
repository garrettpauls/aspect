use chrono::{DateTime, Local};
use conrod_core::{widget, Widget, Sizeable, Positionable, Colorable};
use std::ffi::OsStr;

use crate::data::File;

widget_ids!(struct Ids {
    background,
    name,
    date,
    size
});

pub struct State {
    ids: Ids,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    /// Color of the item's background
    #[conrod(default = "theme.shape_color")]
    pub color: Option<conrod_core::Color>,
    /// Color of the item's label
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<conrod_core::Color>,
    /// Size of the item's font
    #[conrod(default = "theme.font_size_small")]
    pub font_size: Option<conrod_core::FontSize>,
}

#[derive(WidgetCommon)]
pub struct ListItem<'a> {
    #[conrod(common_builder)] common: widget::CommonBuilder,
    style: Style,
    file: &'a File,
}

impl<'a> ListItem<'a> {
    pub fn new(file: &File) -> ListItem {
        ListItem {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            file,
        }
    }

    pub fn with_style(mut self, s: Option<Style>) -> Self {
        if let Some(s) = s {
            self.style = s;
        }
        self
    }
}

impl<'a> Widget for ListItem<'a> {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen)
        }
    }

    fn style(&self) -> Self::Style { self.style.clone() }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            state,
            ui,
            id,
            style,
            ..
        } = args;

        widget::Canvas::new()
            .parent(id)
            .wh_of(id)
            .top_left_of(id)
            .color(style.color(&ui.theme))
            .pad(4.0)
            .set(state.ids.background, ui);

        // TODO: measure strings and layout correctly
        let h = self.get_h(ui).unwrap_or(50.0) / 2.5;
        let name = self.file.path.file_name().unwrap_or_else(|| OsStr::new("")).to_string_lossy();
        widget::Text::new(&name)
            .parent(id)
            .w_of(id).h(h)
            .top_left_of(state.ids.background)
            .left_justify()
            .no_line_wrap()
            .set(state.ids.name, ui);

        let modified: DateTime<Local> = DateTime::from(self.file.last_modified());
        let modified = modified.format("    %F").to_string();
        widget::Text::new(&modified)
            .parent(id)
            .w_of(id).h(h)
            .bottom_left_of(state.ids.background)
            .align_left_of(state.ids.name)
            .left_justify()
            .no_line_wrap()
            .set(state.ids.date, ui);

        let size = self.file.size();
        let size = format!("{}    ", size);
        widget::Text::new(&size)
            .parent(id)
            .w_of(id).h(h)
            .align_left_of(state.ids.date)
            .align_top_of(state.ids.date)
            .right_justify()
            .no_line_wrap()
            .set(state.ids.size, ui);
    }
}