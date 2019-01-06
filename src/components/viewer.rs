use conrod_core::{widget, Widget, Sizeable, Positionable};
use conrod_core::image::Id;

widget_ids!(struct Ids {
    image,
});

pub struct State {
    ids: Ids,
}

#[derive(WidgetCommon)]
pub struct ImageViewer {
    #[conrod(common_builder)] common: widget::CommonBuilder,
    image_id: Id,
}

impl ImageViewer {
    pub fn new(image_id: Id) -> Self {
        ImageViewer {
            common: widget::CommonBuilder::default(),
            image_id,
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

        widget::Image::new(self.image_id)
            .parent(id).graphics_for(id)
            .wh_of(id)
            .top_left()
            .set(state.ids.image, ui);
    }
}
