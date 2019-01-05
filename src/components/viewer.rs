use conrod_core::{widget, Widget, Sizeable, Colorable, Positionable};

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
            .color(ui.theme.label_color)
            .w_of(id).h(ui.theme.font_size_large as f64)
            .center_justify()
            .middle()
            .font_size(ui.theme.font_size_large)
            .set(state.ids.text, ui);
    }
}
