use conrod_core::{widget, Widget, Sizeable, Colorable, Positionable};
use conrod_core::event::Button;
use conrod_core::input::{Key, MouseButton};

use super::{ActionOverlay, ImageViewer};
use crate::data::FileList;
use crate::res::Resources;
use crate::systems::EventSystem;

widget_ids!(struct Ids {
    background,
    overlay,
    viewer,
    file_nav,
});

pub struct State {
    ids: Ids,
    is_overlay_visible: bool,
}

#[derive(WidgetCommon)]
pub struct App<'a> {
    #[conrod(common_builder)] common: widget::CommonBuilder,
    res: &'a Resources,
    events: &'a mut EventSystem,
    files: &'a Option<FileList>,
}

impl<'a> App<'a> {
    pub fn new(events: &'a mut EventSystem, res: &'a Resources, files: &'a Option<FileList>) -> Self {
        App {
            common: widget::CommonBuilder::default(),
            res,
            events,
            files,
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

        widget::Canvas::new()
            .parent(id).graphics_for(id)
            .color(ui.theme.background_color)
            .wh_of(id)
            .set(state.ids.background, ui);

        self.process_input(ui, state, id);

        if let Some(files) = &self.files {
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
        use crate::systems::events::Nav;
        let releases = ui.widget_input(id).releases().chain(ui.widget_input(state.ids.viewer).releases());
        for release in releases {
            match release.button {
                Button::Keyboard(Key::Space) | Button::Mouse(MouseButton::Middle, _) =>
                    state.update(|s| s.is_overlay_visible = !s.is_overlay_visible),
                Button::Mouse(MouseButton::Button6, _) | Button::Keyboard(Key::Right) => self.events.push(Nav::ImageNext.into()),
                Button::Mouse(MouseButton::X2, _) | Button::Keyboard(Key::Left) => self.events.push(Nav::ImagePrev.into()),
                _ => (),
            }
        }
    }
}
