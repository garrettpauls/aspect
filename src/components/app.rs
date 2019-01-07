use conrod_core::{widget, Widget, Sizeable, Colorable, Positionable};
use conrod_core::event::Button;
use conrod_core::input::{Key, MouseButton};
use std::path::PathBuf;

use super::{Action, ActionOverlay, ImageViewer, ImageData};
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
    current_file_path: Option<PathBuf>,
}

#[derive(WidgetCommon)]
pub struct App {
    #[conrod(common_builder)] common: widget::CommonBuilder,
    image: Option<ImageData>,
}

impl App {
    pub fn new(image: Option<ImageData>) -> Self {
        App {
            common: widget::CommonBuilder::default(),
            image,
        }
    }
}

impl Widget for App {
    type State = State;
    type Style = ();
    type Event = Vec<Action>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            is_overlay_visible: false,
            files: FileList::from_environment(),
            current_file_path: None,
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

        let mut actions = Vec::new();

        if let Some(files) = &state.files {
            if let Some(image) = self.image {
                ImageViewer::new(image)
                    .parent(id).graphics_for(id)
                    .wh_of(id)
                    .set(state.ids.viewer, ui);
            }

            if state.is_overlay_visible {
                actions.append(&mut ActionOverlay::new(&files)
                    .parent(id)
                    .wh_of(id)
                    .set(state.ids.overlay, ui));
            }

            for release in ui.widget_input(id).releases() {
                match release.button {
                    Button::Keyboard(Key::Space) | Button::Mouse(MouseButton::Middle, _) =>
                        state.update(|s| s.is_overlay_visible = !s.is_overlay_visible),
                    Button::Mouse(MouseButton::Button6, _) => actions.push(Action::ImageNext),
                    Button::Mouse(MouseButton::X2, _) => actions.push(Action::ImagePrev),
                    _ => (),
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

        let mut results = Vec::new();

        for action in actions {
            log::info!("overlay action: {:?}", action);

            match action {
                Action::ImageNext => state.update(|s| if let Some(f) = &mut s.files { f.increment_current(1) }),
                Action::ImagePrev => state.update(|s| if let Some(f) = &mut s.files { f.increment_current(-1) }),
                Action::Select(i) => state.update(|s| if let Some(f) = &mut s.files { f.set_current(i) }),
                Action::Sort(srt) => state.update(|s| if let Some(f) = &mut s.files { f.sort_by(srt) }),
                unhandled => results.push(unhandled),
            }
        }

        let new_file = if let Some(f) = &state.files {
            f.current().map(|f| f.path.clone())
        } else { None };
        if state.current_file_path != new_file {
            if let Some(file) = &new_file {
                results.push(Action::LoadImage(file.clone()));
            }

            state.update(|s| s.current_file_path = new_file);
        }


        results
    }
}
