use conrod_core::{widget, Widget};

widget_ids!(struct Ids {

});

pub struct State {
    ids: Ids,
}

#[derive(WidgetCommon)]
pub struct Template {
    #[conrod(common_builder)] common: widget::CommonBuilder,
}

impl Template {
    pub fn new() -> Template {
        Template {
            common: widget::CommonBuilder::default()
        }
    }
}

impl Widget for Template {
    type State = State;
    type Style = ();
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen)
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
    }
}