use conrod_core::{widget, Widget, Sizeable, Positionable};
use crate::components::ImageData;

widget_ids!(struct Ids {
    image,
});

pub struct State {
    ids: Ids,
}

#[derive(WidgetCommon)]
pub struct ImageViewer {
    #[conrod(common_builder)] common: widget::CommonBuilder,
    image: ImageData,
}

impl ImageViewer {
    pub fn new(image: ImageData) -> Self {
        ImageViewer {
            common: widget::CommonBuilder::default(),
            image,
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

        let [uw, uh] = ui.wh_of(id).unwrap_or(ui.window_dim());
        let w = self.image.w as f64;
        let h = self.image.h as f64;
        let scale = (uw / w).min(uh / h);
        let w = scale * w;
        let h = scale * h;

        widget::Image::new(self.image.id)
            .parent(id).graphics_for(id)
            .w_h(w, h)
            .top_left_with_margins((uh - h) / 2.0, (uw - w) / 2.0)
            .set(state.ids.image, ui);
    }
}
