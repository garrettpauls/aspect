use conrod_core::{widget, Widget, Sizeable, color, Colorable, Positionable};

widget_ids!(struct Ids {
    text,
});

pub struct State {
    ids: Ids,
}

#[derive(WidgetCommon)]
pub struct ImageViewer {
    #[conrod(common_builder)] common: widget::CommonBuilder,
}

impl ImageViewer {
    pub fn new() -> Self {
        ImageViewer {
            common: widget::CommonBuilder::default(),
        }
    }
}

impl Widget for ImageViewer {
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

        widget::Text::new("Image Viewer")
            .parent(id)
            .color(color::WHITE)
            .w_of(id).h(50.0)
            .center_justify()
            .middle()
            .font_size(30)
            .set(state.ids.text, ui);
    }
}
