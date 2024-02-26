#![allow(unused)]
use chrono::{Datelike, Month, NaiveDate, Weekday};
use floem::{
    id::Id,
    kurbo,
    peniko::Color,
    reactive::{create_rw_signal, create_updater, RwSignal},
    style_class,
    taffy::{style_helpers::*, *},
    unit::{PxPctAuto, UnitExt},
    view::{default_compute_layout, AnyView, AnyWidget, View, ViewData, Widget},
    views::{
        container, dyn_stack, empty, label, stack, stack_from_iter, svg, text, Decorators,
        DynStack, List,
    },
    widgets::dropdown::dropdown,
};

struct CalendarDays {
    start_blanks: u8,
    days: u8,
    end_blanks: u8,
}
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
enum DayKind {
    Blank,
    Day(u8),
}
impl IntoIterator for CalendarDays {
    type Item = DayKind;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        (0..self.start_blanks)
            .map(|_val| DayKind::Blank)
            .chain((1..=self.days).map(DayKind::Day))
            .chain((0..self.end_blanks).map(|_val| DayKind::Blank))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

struct CalendarDay {
    view_data: ViewData,
    child: AnyWidget,
    selected: bool,
}
impl View for CalendarDay {
    fn view_data(&self) -> &ViewData {
        &self.view_data
    }

    fn view_data_mut(&mut self) -> &mut ViewData {
        &mut self.view_data
    }

    fn build(self) -> AnyWidget {
        Box::new(self)
    }
}
impl Widget for CalendarDay {
    fn view_data(&self) -> &ViewData {
        &self.view_data
    }

    fn view_data_mut(&mut self) -> &mut ViewData {
        &mut self.view_data
    }

    fn for_each_child<'a>(&'a self, for_each: &mut dyn FnMut(&'a dyn Widget) -> bool) {
        for_each(&self.child);
    }

    fn for_each_child_mut<'a>(&'a mut self, for_each: &mut dyn FnMut(&'a mut dyn Widget) -> bool) {
        for_each(&mut self.child);
    }

    fn for_each_child_rev_mut<'a>(
        &'a mut self,
        for_each: &mut dyn FnMut(&'a mut dyn Widget) -> bool,
    ) {
        for_each(&mut self.child);
    }

    fn style(&mut self, cx: &mut floem::context::StyleCx<'_>) {
        if self.selected {
            cx.save();
            cx.selected();
            cx.style_view(&mut self.child);
            cx.restore();
        } else {
            cx.style_view(&mut self.child);
        }
        cx.selected();
    }
    fn update(&mut self, cx: &mut floem::context::UpdateCx, state: Box<dyn std::any::Any>) {
        if let Ok(state) = state.downcast::<bool>() {
            self.selected = *state;
            cx.app_state_mut().request_style_recursive(self.id());
        }
    }
}
fn calendar_day(widget: impl View) -> CalendarDay {
    CalendarDay {
        view_data: ViewData::new(Id::next()),
        child: widget.build(),
        selected: false,
    }
}

style_class!(pub DateContainerClass);
style_class!(pub DateClass);

pub enum DateMessage {
    StartDate(NaiveDate),
    EndDate(NaiveDate),
}

pub struct DatePicker {
    view_data: ViewData,
    start_date: NaiveDate,
    end_date: NaiveDate,
    child: Box<dyn Widget>,
}
impl View for DatePicker {
    fn view_data(&self) -> &ViewData {
        &self.view_data
    }

    fn view_data_mut(&mut self) -> &mut ViewData {
        &mut self.view_data
    }

    fn build(self) -> AnyWidget {
        Box::new(self.keyboard_navigatable())
    }
}

impl Widget for DatePicker {
    fn view_data(&self) -> &ViewData {
        &self.view_data
    }

    fn view_data_mut(&mut self) -> &mut ViewData {
        &mut self.view_data
    }
    fn for_each_child<'a>(&'a self, for_each: &mut dyn FnMut(&'a dyn Widget) -> bool) {
        for_each(&self.child);
    }

    fn for_each_child_mut<'a>(&'a mut self, for_each: &mut dyn FnMut(&'a mut dyn Widget) -> bool) {
        for_each(&mut self.child);
    }

    fn for_each_child_rev_mut<'a>(
        &'a mut self,
        for_each: &mut dyn FnMut(&'a mut dyn Widget) -> bool,
    ) {
        for_each(&mut self.child);
    }

    fn update(&mut self, _cx: &mut floem::context::UpdateCx, state: Box<dyn std::any::Any>) {
        if let Ok(message) = state.downcast::<DateMessage>() {
            match *message {
                DateMessage::StartDate(date) => self.start_date = date,
                DateMessage::EndDate(date) => self.end_date = date,
            }
        }
    }

    fn compute_layout(&mut self, cx: &mut floem::context::ComputeLayoutCx) -> Option<kurbo::Rect> {
        let layout = cx.get_layout(self.id()).unwrap();

        default_compute_layout(&mut self.child, cx)
    }
}

pub fn datepicker(
    start_date: impl Fn() -> NaiveDate + 'static,
    end_date: impl Fn() -> NaiveDate + 'static,
) -> DatePicker {
    let id = Id::next();
    let start_date = create_updater(start_date, move |new_date| {
        id.update_state(DateMessage::StartDate(new_date))
    });
    let end_date = create_updater(end_date, move |new_date| {
        id.update_state(DateMessage::EndDate(new_date))
    });

    let active_month = create_rw_signal(start_date.month());

    let child = dyn_stack(
        move || {
            let num_begin_blanks = start_date.with_day(1).unwrap().weekday().day_num();
            let num_days = get_days_in_month(start_date);
            let num_end_blanks = 6 - start_date
                .with_day(num_days as u32)
                .unwrap()
                .weekday()
                .day_num();
            CalendarDays {
                start_blanks: num_begin_blanks,
                days: num_days as u8,
                end_blanks: num_end_blanks,
            }
        },
        |val| *val,
        |val| match val {
            DayKind::Blank => empty().class(DateContainerClass).style(|s| s).any(),
            DayKind::Day(num) => {
                let inner = calendar_day(text(num).style(|s| s.size_full()).class(DateClass))
                    .style(|s| s.aspect_ratio(1.).items_center().justify_center());
                let inner_id = inner.id();
                container(inner)
                    .style(|s| s.items_center().justify_center())
                    .keyboard_navigatable()
                    .on_event_stop(floem::event::EventListener::FocusGained, move |_| {
                        inner_id.request_focus();
                        inner_id.update_state(());
                    })
                    .on_click_stop(move |_| {
                        dbg!("clicked");
                        inner_id.request_focus();
                        inner_id.update_state(true);
                    })
                    .class(DateContainerClass)
                    .any()
            }
        },
    )
    .style(|s| {
        s.grid()
            .grid_template_columns(vec![repeat(GridTrackRepetition::Count(7), vec![fr(1.)])])
            .grid_auto_rows(vec![minmax(min_content(), auto())])
            .size_full()
    })
    .build();

    DatePicker {
        view_data: ViewData::new(id),
        start_date,
        end_date,
        child,
    }
    .style(|s| s)
}
trait WeekDayNumExt {
    fn day_num(&self) -> u8;
}
impl WeekDayNumExt for Weekday {
    fn day_num(&self) -> u8 {
        match self {
            Weekday::Sun => 0,
            Weekday::Mon => 1,
            Weekday::Tue => 2,
            Weekday::Wed => 3,
            Weekday::Thu => 4,
            Weekday::Fri => 5,
            Weekday::Sat => 6,
        }
    }
}

fn get_days_in_month(date: NaiveDate) -> u32 {
    let year = date.year();
    let month = date.month();

    // Calculate the first day of the next month
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };

    // Get the last day of the given month by subtracting one day from the first day of the next month
    let last_day_of_month = next_month_date.pred_opt().unwrap();

    // Return the day, which represents the number of days in the month
    last_day_of_month.day()
}
