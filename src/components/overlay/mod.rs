use conrod_core::{color, widget, Labelable, Positionable, Sizeable, Widget};
use std::time::Duration;

use crate::data::{FileList, Rating, FILE_SORT_METHODS};
use crate::res::Resources;
use crate::systems::{events as e, AppEvent, EventSystem};

mod list_item;
mod rating;

widget_ids!(struct Ids {
    next,
    prev,
    sort,
    file_list,
    filter_text,
    filter_rating,
    rating,
    bg_list,
    slideshow,
});

pub struct State {
    ids: Ids,
    filter_text: String,
    filter_rating: Option<Rating>,
}

#[derive(WidgetCommon)]
pub struct ActionOverlay<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    files: &'a FileList,
    res: &'a Resources,
    events: &'a mut EventSystem,
}

impl<'a> ActionOverlay<'a> {
    pub fn new(files: &'a FileList, res: &'a Resources, events: &'a mut EventSystem) -> Self {
        ActionOverlay {
            common: widget::CommonBuilder::default(),
            files,
            res,
            events,
        }
    }
}

impl<'a> Widget for ActionOverlay<'a> {
    type State = State;
    type Style = ();
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            filter_text: "".to_owned(),
            filter_rating: None,
        }
    }

    fn style(&self) -> Self::Style {}

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            state,
            ui,
            id,
            rect,
            ..
        } = args;
        const ACTION_HEIGHT: f64 = 48.0;

        widget::Rectangle::fill_with([300.0, rect.h()], ui.theme.shape_color)
            .parent(id)
            .graphics_for(id)
            .top_right()
            .set(state.ids.bg_list, ui);

        for event in widget::TextBox::new(&state.filter_text)
            .parent(id)
            .w_h(300.0, ACTION_HEIGHT)
            .top_right_of(state.ids.bg_list)
            .set(state.ids.filter_text, ui)
        {
            use conrod_core::widget::text_box::Event;
            match event {
                Event::Update(str) => state.update(|s| s.filter_text = str),
                Event::Enter => self
                    .events
                    .push(e::Filter::Text(state.filter_text.clone()).into()),
            }
        }

        if let Some(filter_rating) = rating::StarRating::new(state.filter_rating.clone(), self.res)
            .parent(id)
            .w_h(300.0, ACTION_HEIGHT)
            .align_left_of(state.ids.filter_text)
            .down_from(state.ids.filter_text, 0.0)
            .set(state.ids.filter_rating, ui)
        {
            if state.filter_rating != filter_rating {
                self.events
                    .push(e::Filter::Rating(filter_rating.clone()).into());
                state.update(|s| s.filter_rating = filter_rating);
            }
        }

        let (mut events, scrollbar) = widget::ListSelect::single(self.files.len())
            .parent(id)
            .flow_down()
            .item_size(50.0)
            .scrollbar_next_to()
            .w_of(state.ids.filter_text)
            .h(ui.h_of(id).unwrap_or(ui.win_h) - ACTION_HEIGHT - ACTION_HEIGHT)
            .down_from(state.ids.filter_rating, 0.0)
            .set(state.ids.file_list, ui);
        while let Some(event) = events.next(ui, |i| self.files.current_index() == i) {
            use conrod_core::widget::list_select::Event;

            match event {
                Event::Item(item) => {
                    if let Some(file) = self.files.get_file(item.i) {
                        let is_selected = self.files.current_index() == item.i;
                        let style = if is_selected {
                            Some(build_selected_style())
                        } else {
                            None
                        };
                        let widget = list_item::ListItem::new(file).with_style(style);
                        item.set(widget, ui);
                    }
                }
                Event::Selection(i) => self.events.push(e::Nav::ImageIndex(i).into()),
                _ => (),
            }
        }
        if let Some(s) = scrollbar {
            s.set(ui);
        }

        for _click in widget::Button::new()
            .parent(id)
            .left_from(state.ids.filter_text, 0.0)
            .align_top_of(state.ids.filter_text)
            .w_h(48.0, ACTION_HEIGHT)
            .label(">>")
            .set(state.ids.next, ui)
        {
            self.events.push(e::Nav::ImageNext.into());
        }

        for _click in widget::Button::new()
            .parent(id)
            .left_from(state.ids.next, 0.0)
            .align_top_of(state.ids.next)
            .wh_of(state.ids.next)
            .label("<<")
            .set(state.ids.prev, ui)
        {
            self.events.push(e::Nav::ImagePrev.into());
        }

        let idx = FILE_SORT_METHODS
            .iter()
            .position(|&x| x == *self.files.current_sort());
        if let Some(new_idx) = widget::DropDownList::new(FILE_SORT_METHODS, idx)
            .parent(id)
            .left_from(state.ids.prev, 0.0)
            .align_top_of(state.ids.next)
            .h_of(state.ids.next)
            .w(192.0)
            .set(state.ids.sort, ui)
        {
            if Some(new_idx) != idx {
                if let Some(method) = FILE_SORT_METHODS.get(new_idx) {
                    self.events.push(AppEvent::Sort(*method));
                }
            }
        }

        let current = self.files.current();
        if let Some(rating) =
            rating::StarRating::new(current.and_then(|f| f.rating.clone()), self.res)
                .parent(id)
                .align_left_of(state.ids.sort)
                .down_from(state.ids.sort, 0.0)
                .w_of(state.ids.sort)
                .h(48.0)
                .set(state.ids.rating, ui)
        {
            self.events.push(e::SetMeta::Rating(rating).into());
        }

        for enabled in widget::Toggle::new(self.files.is_slideshow_enabled())
            .parent(id)
            .left_from(state.ids.rating, 0.0)
            .align_top_of(state.ids.sort)
            .w_h(192.0, 48.0)
            .label(if self.files.is_slideshow_enabled() {
                "Slideshow Running"
            } else {
                "Start Slideshow"
            })
            .set(state.ids.slideshow, ui)
        {
            self.events.push(
                if enabled {
                    e::Slideshow::Start(Duration::from_secs(15))
                } else {
                    e::Slideshow::Stop
                }
                .into(),
            );
        }
    }
}

fn build_selected_style() -> list_item::Style {
    let mut style = list_item::Style::default();
    style.label_color = Some(color::DARK_RED);
    style.color = Some(color::CHARCOAL);
    style
}
