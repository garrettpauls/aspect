use super::{ActionOverlay, ImageViewer};
use conrod_core::{widget, Widget, Sizeable, Colorable};
use conrod_core::event::{Event, Ui, Release, Button};
use conrod_core::input::Key;

widget_ids!(struct Ids {
    canvas,
    overlay,
    viewer
});

pub struct State {
    ids: Ids,
    is_overlay_visible: bool,
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

        self.process_events(state, ui);

        widget::Canvas::new()
            .parent(id)
            .color(ui.theme.background_color)
            .wh_of(id)
            .set(state.ids.canvas, ui);

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
