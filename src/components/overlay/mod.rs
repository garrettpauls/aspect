use conrod_core::{color, widget, Widget, Sizeable, Positionable, Labelable};

use crate::data::FileList;

mod list_item;

widget_ids!(struct Ids {
    next,
    prev,
    file_list
});

#[derive(Copy, Clone, Debug)]
pub enum Action {
    ImageNext,
    ImagePrev,
    Select(usize),
}

pub struct State {
    ids: Ids,
}

#[derive(WidgetCommon)]
pub struct ActionOverlay<'a> {
    #[conrod(common_builder)] common: widget::CommonBuilder,
    files: &'a FileList,
}

impl<'a> ActionOverlay<'a> {
    pub fn new(files: &'a FileList) -> Self {
        ActionOverlay {
            common: widget::CommonBuilder::default(),
            files,
        }
    }
}

impl<'a> Widget for ActionOverlay<'a> {
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
        let mut actions = Vec::new();

        let (mut events, scrollbar) =
            widget::ListSelect::single(self.files.files.len())
                .parent(id)
                .flow_down()
                .item_size(50.0)
                .scrollbar_next_to()
                .w(300.0).h_of(id)
                .top_right()
                .set(state.ids.file_list, ui);
        while let Some(event) = events.next(ui, |i| self.files.current_index() == i) {
            use conrod_core::widget::list_select::Event;

            match event {
                Event::Item(item) => {
                    if let Some(file) = self.files.get_file(item.i) {
                        let is_selected = self.files.current_index() == item.i;
                        let style = if is_selected { Some(build_selected_style()) } else { None };
                        let widget = list_item::ListItem::new(file).with_style(style);
                        item.set(widget, ui);
                    }
                }
                Event::Selection(i) => actions.push(Action::Select(i)),
                _ => (),
            }
        }
        if let Some(s) = scrollbar {
            s.set(ui);
        }

        for _click in widget::Button::new()
            .parent(id)
            .left_from(state.ids.file_list, 0.0)
            .align_top_of(state.ids.file_list)
            .w_h(48.0, 48.0)
            .label(">>")
            .set(state.ids.next, ui) {
            actions.push(Action::ImageNext);
        }

        for _click in widget::Button::new()
            .parent(id)
            .left_from(state.ids.next, 0.0)
            .align_top_of(state.ids.next)
            .w_h(48.0, 48.0)
            .label("<<")
            .set(state.ids.prev, ui) {
            actions.push(Action::ImagePrev);
        }

        actions
    }
}

fn build_selected_style() -> list_item::Style {
    let mut style = list_item::Style::default();
    style.label_color = Some(color::DARK_RED);
    style.color = Some(color::CHARCOAL);
    style
}