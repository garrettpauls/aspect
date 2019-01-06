use conrod_core::{widget, Widget, Sizeable, Colorable, Positionable};
use conrod_core::event::Button;
use conrod_core::input::{Key, MouseButton};

use super::{ActionOverlay, ImageViewer};
use super::overlay::Action;
use crate::data::FileList;

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
}

#[derive(WidgetCommon)]
pub struct App {
    #[conrod(common_builder)] common: widget::CommonBuilder,
}

impl App {
    pub fn new() -> Self {
        App {
            common: widget::CommonBuilder::default(),
        }
    }
}

impl Widget for App {
    type State = State;
    type Style = ();
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            is_overlay_visible: false,
            files: FileList::from_environment(),
        }
    }

    fn style(&self) -> Self::Style {}

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
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

        if let Some(files) = &state.files {
            ImageViewer::new()
                .parent(id)
                .wh_of(id)
                .set(state.ids.viewer, ui);

            let mut actions = if state.is_overlay_visible {
                ActionOverlay::new(&files)
                    .parent(id)
                    .wh_of(id)
                    .set(state.ids.overlay, ui)
            } else { Vec::new() };

            for release in ui.widget_input(id).releases() {
                match release.button {
                    Button::Keyboard(Key::Space) | Button::Mouse(MouseButton::Middle, _) =>
                        state.update(|s| s.is_overlay_visible = !s.is_overlay_visible),
                    Button::Mouse(MouseButton::Button6, _) => actions.push(Action::ImageNext),
                    Button::Mouse(MouseButton::X2, _) => actions.push(Action::ImagePrev),
                    _ => (),
                }
            }

            for action in actions {
                log::info!("overlay action: {:?}", action);

                match action {
                    Action::ImageNext => state.update(|s| if let Some(f) = &mut s.files { f.increment_current(1) }),
                    Action::ImagePrev => state.update(|s| if let Some(f) = &mut s.files { f.increment_current(-1) }),
                    Action::Select(i) => state.update(|s| if let Some(f) = &mut s.files { f.set_current(i) }),
                    Action::Sort(srt) => state.update(|s| if let Some(f) = &mut s.files { f.sort_by(srt) }),
                }
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
