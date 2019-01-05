use conrod_core::{widget, Widget, Sizeable, Colorable};
use super::{ActionOverlay, ImageViewer};

widget_ids!(struct Ids {
    canvas,
    overlay,
    viewer
});

pub struct State {
    ids: Ids,
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
            .set(state.ids.canvas, ui);

        ImageViewer::new()
            .parent(id)
            .wh_of(id)
            .set(state.ids.viewer, ui);

        ActionOverlay::new()
            .parent(id)
            .wh_of(id)
            .set(state.ids.overlay, ui);
    }
}
