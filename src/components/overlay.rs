use conrod_core::{widget, Widget, Sizeable, Positionable, Labelable};

widget_ids!(struct Ids {
    next,
    prev
});

#[derive(Copy, Clone, Debug)]
pub enum Action {
    ImageNext,
    ImagePrev,
}

pub struct State {
    ids: Ids,
}

#[derive(WidgetCommon)]
pub struct ActionOverlay {
    #[conrod(common_builder)] common: widget::CommonBuilder,
}

impl ActionOverlay {
    pub fn new() -> Self {
        ActionOverlay {
            common: widget::CommonBuilder::default(),
        }
    }
}

impl Widget for ActionOverlay {
    type State = State;
    type Style = ();
    type Event = Vec<Action>;

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
        let mut events = Vec::new();

        for _click in widget::Button::new()
            .parent(id)
            .top_right()
            .w_h(48.0, 48.0)
            .label(">>")
            .set(state.ids.next, ui) {
            events.push(Action::ImageNext);
        }

        for _click in widget::Button::new()
            .parent(id)
            .left_from(state.ids.next, 0.0)
            .align_top_of(state.ids.next)
            .w_h(48.0, 48.0)
            .label("<<")
            .set(state.ids.prev, ui) {
            events.push(Action::ImagePrev);
        }

        events
    }
}
