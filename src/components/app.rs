use conrod_core::{widget, Widget, Sizeable, Colorable, Positionable};
use conrod_core::event::Button;
use conrod_core::input::{Key, MouseButton};
use std::path::PathBuf;

use super::{ActionOverlay, ImageViewer};
use crate::data::FileList;
use crate::res::Resources;
use crate::systems::{EventSystem, events::AppEvent, events as e};

widget_ids!(struct Ids {
    background,
    overlay,
    viewer,
    file_nav,
});

pub struct State {
    ids: Ids,
    is_overlay_visible: bool,
    files: Option<FileList>,
    current_file_path: Option<PathBuf>,
}

#[derive(WidgetCommon)]
pub struct App<'a> {
    #[conrod(common_builder)] common: widget::CommonBuilder,
    res: &'a Resources,
    events: &'a mut EventSystem,
}

impl<'a> App<'a> {
    pub fn new(events: &'a mut EventSystem, res: &'a Resources) -> Self {
        App {
            common: widget::CommonBuilder::default(),
            res,
            events,
        }
    }
}

impl<'a> Widget for App<'a> {
    type State = State;
    type Style = ();
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            is_overlay_visible: false,
            files: FileList::from_environment(),
            current_file_path: None,
        }
    }

    fn style(&self) -> Self::Style {}

    fn update(mut self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            state,
            ui,
            id,
            ..
        } = args;

        for event in self.events.events() {
            match event {
                AppEvent::Nav(nav) => match nav {
                    e::Nav::ImageNext => state.update(|s| if let Some(f) = &mut s.files { f.next() }),
                    e::Nav::ImagePrev => state.update(|s| if let Some(f) = &mut s.files { f.prev() }),
                    e::Nav::ImageIndex(i) => state.update(|s| if let Some(f) = &mut s.files { f.set_current(*i) }),
                },
                AppEvent::Sort(srt) => state.update(|s| if let Some(f) = &mut s.files { f.sort_by(*srt) }),
                AppEvent::Filter(filter) => match filter {
                    e::Filter::Text(txt) => state.update(|s| if let Some(f) = &mut s.files {
                        let new = f.get_filter().clone().with_name(&txt);
                        f.apply_filter(new)
                    }),
                    e::Filter::Rating(rating) => state.update(|s| if let Some(f) = &mut s.files {
                        let new = f.get_filter().clone().with_rating(&rating);
                        f.apply_filter(new)
                    }),
                },
                AppEvent::SetMeta(meta) => match meta {
                    e::SetMeta::Rating(rating) => state.update(|s| if let Some(f) = &mut s.files { f.set_rating(rating.clone()) }),
                },
                _ => (),
            }
        }

        let new_file = if let Some(f) = &state.files {
            f.current().map(|f| f.path.clone())
        } else { None };
        if state.current_file_path != new_file {
            if let Some(file) = &new_file {
                self.events.push(AppEvent::Load(file.to_path_buf()));
            }

            state.update(|s| s.current_file_path = new_file);
        }

        widget::Canvas::new()
            .parent(id).graphics_for(id)
            .color(ui.theme.background_color)
            .wh_of(id)
            .set(state.ids.background, ui);

        self.process_input(ui, state, id);

        if let Some(files) = &state.files {
            ImageViewer::new(self.events)
                .parent(id)
                .wh_of(id)
                .set(state.ids.viewer, ui);

            if state.is_overlay_visible {
                ActionOverlay::new(&files, self.res, self.events)
                    .parent(id)
                    .wh_of(id)
                    .set(state.ids.overlay, ui);
            }
        } else {
            widget::Text::new("Rerun the program with an argument pointing to a directory or file.\nPicking a file from here may be supported in the future.")
                .parent(id).graphics_for(id)
                .padded_wh_of(id, 24.0)
                .top_left()
                .center_justify()
                .wrap_by_word()
                .font_size(ui.theme.font_size_large)
                .set(state.ids.file_nav, ui);
        }
    }
}

impl<'a> App<'a> {
    fn process_input(&mut self, ui: &mut conrod_core::UiCell, state: &mut widget::State<State>, id: widget::Id) {
        let releases = ui.widget_input(id).releases().chain(ui.widget_input(state.ids.viewer).releases());
        for release in releases {
            match release.button {
                Button::Keyboard(Key::Space) | Button::Mouse(MouseButton::Middle, _) =>
                    state.update(|s| s.is_overlay_visible = !s.is_overlay_visible),
                Button::Mouse(MouseButton::Button6, _) | Button::Keyboard(Key::Right) => self.events.push(e::Nav::ImageNext.into()),
                Button::Mouse(MouseButton::X2, _) | Button::Keyboard(Key::Left) => self.events.push(e::Nav::ImagePrev.into()),
                _ => (),
            }
        }
    }
}
