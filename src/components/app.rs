use super::{ActionOverlay, ImageViewer};
use super::super::data::FileList;
use conrod_core::{widget, Widget, Sizeable, Colorable, Positionable};
use conrod_core::event::{Event, Ui, Release, Button};
use conrod_core::input::Key;

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
            .parent(id)
            .color(ui.theme.background_color)
            .wh_of(id)
            .set(state.ids.background, ui);

        if let Some(_files) = &state.files {
            self.process_events(state, ui);

            ImageViewer::new()
                .parent(id)
                .wh_of(id)
                .set(state.ids.viewer, ui);

            if state.is_overlay_visible {
                for action in ActionOverlay::new()
                    .parent(id)
                    .wh_of(id)
                    .set(state.ids.overlay, ui) {
                    log::info!("overlay action: {:?}", action);
                }
            }
        } else {
            widget::Text::new("Rerun the program with an argument pointing to a directory or file.\nPicking a file from here may be supported in the future.")
                .parent(id)
                .padded_wh_of(id, 24.0)
                .top_left()
                .center_justify()
                .wrap_by_word()
                .font_size(ui.theme.font_size_large)
                .set(state.ids.file_nav, ui);
        }
    }
}

impl App {
    fn process_events(&self, state: &mut widget::State<State>, ui: &mut conrod_core::UiCell) {
        for event in ui.global_input().events().filter_map(|e| match e {
            Event::Ui(ui) => Some(ui),
            _ => None
        }) {
            match event {
                Ui::Release(_, Release { button, .. }) => match button {
                    Button::Keyboard(Key::Space) => {
                        state.update(|s| s.is_overlay_visible = !s.is_overlay_visible);
                    }
                    _ => ()
                },
                _ => ()
            }
        }
    }
}
