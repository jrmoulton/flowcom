use std::ops::{Add, Sub};

use floem::{
    prop, prop_extractor,
    reactive::{create_memo, RwSignal, SignalGet, SignalUpdate},
    style::{CustomStylable, Style, StylePropValue},
    style_class,
    taffy::{style_helpers::*, GridTrackRepetition},
    views::{container, dyn_stack, text, Decorators, VirtualVector},
    IntoView, View, ViewId,
};
use jiff::{
    civil::{Date, Weekday},
    ToSpan,
};

#[derive(Clone, Copy, PartialEq)]
pub struct CalendarDays {
    pub start_date: Date,
    pub end_date: Date,
}
impl CalendarDays {
    pub fn with_show_minimum(self, show_minimum: ShowMinimum) -> Self {
        match show_minimum {
            ShowMinimum::Day { .. } => self,
            ShowMinimum::Week => {
                let sunday = if self.start_date.weekday() == Weekday::Sunday {
                    self.start_date
                } else {
                    self.start_date.nth_weekday(-1, Weekday::Sunday).unwrap()
                };
                let saturday = if self.end_date.weekday() == Weekday::Saturday {
                    self.end_date
                } else {
                    self.end_date.nth_weekday(1, Weekday::Saturday).unwrap()
                };
                CalendarDays {
                    start_date: sunday,
                    end_date: saturday,
                }
            }
            ShowMinimum::Month { .. } => {
                let first_of_month = self.start_date.first_of_month();
                let first_calendar_day = if first_of_month.weekday() == Weekday::Sunday {
                    first_of_month
                } else {
                    first_of_month.nth_weekday(-1, Weekday::Sunday).unwrap()
                };
                let last_of_month = self.end_date.last_of_month();
                let last_calendar_day = if last_of_month.weekday() == Weekday::Saturday {
                    last_of_month
                } else {
                    last_of_month.nth_weekday(1, Weekday::Saturday).unwrap()
                };
                CalendarDays {
                    start_date: first_calendar_day,
                    end_date: last_calendar_day,
                }
            }
            ShowMinimum::Year => {
                todo!()
            }
        }
    }
}
impl Add<i32> for CalendarDays {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        CalendarDays {
            start_date: self.start_date,
            end_date: self.end_date + rhs.days(),
        }
    }
}
impl Iterator for CalendarDays {
    type Item = Date;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_date <= self.end_date {
            let next = self.start_date;
            self.start_date += 1.day();
            Some(next)
        } else {
            None
        }
    }
}

impl VirtualVector<Date> for CalendarDays {
    fn total_len(&self) -> usize {
        self.end_date.sub(self.start_date).get_days().abs() as usize
    }

    fn slice(&mut self, range: std::ops::Range<usize>) -> impl Iterator<Item = Date> {
        let start_date = self
            .start_date
            .add(jiff::Span::new().days(range.start as i64));
        let end_date = start_date.add(jiff::Span::new().days((range.end - range.start) as i64));
        CalendarDays {
            start_date,
            end_date,
        }
    }
}

style_class!(pub Today);
style_class!(pub Weekend);
style_class!(pub OtherMonth);

pub struct CalendarDay {
    view_id: ViewId,
    _date: Date,
}
impl View for CalendarDay {
    fn id(&self) -> ViewId {
        self.view_id
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        format!("Calendar Day: {}", self._date).into()
    }
}
impl CalendarDay {
    pub fn new<V: IntoView + 'static>(date: Date, widget: V) -> Self {
        let id = ViewId::new();
        let widget = widget.into_view();
        id.add_child(widget.into_any());
        CalendarDay {
            view_id: id,
            _date: date,
        }
    }

    pub fn basic(date: Date) -> Self {
        Self::new(
            date,
            container(text(date.day()))
                .style(|s| s.items_center().justify_center())
                .class(DateContainerClass),
        )
    }
}

style_class!(pub DateContainerClass);
style_class!(pub DateClass);
style_class!(pub DatePickerClass);
style_class!(pub DateGridClass);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ShowMinimum {
    Day,
    Week,
    #[default]
    Month,
    Year,
}
impl StylePropValue for ShowMinimum {
    fn debug_view(&self) -> Option<Box<dyn View>> {
        Some(text(format!("{:?}", self)).into_any())
    }

    fn interpolate(&self, _other: &Self, _value: f64) -> Option<Self> {
        None
    }
}
prop!(pub ShowMin: ShowMinimum {} = Default::default());

prop_extractor! {
    DatePickerStyle {
        show_minimum: ShowMin,
    }
}

pub struct Calendar {
    id: ViewId,
    show_minimum: RwSignal<ShowMinimum>,
    style: DatePickerStyle,
}

impl View for Calendar {
    fn id(&self) -> ViewId {
        self.id
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "Date Picker".into()
    }

    fn style_pass(&mut self, cx: &mut floem::context::StyleCx<'_>) {
        if self.style.read(cx) {
            self.show_minimum.set(self.style.show_minimum());
        }
        for child in self.id.children() {
            cx.style_view(child);
        }
    }

    // fn compute_layout(
    //     &mut self,
    //     cx: &mut floem::context::ComputeLayoutCx,
    // ) -> Option<floem::kurbo::Rect> {
    // }
}

impl Calendar {
    pub fn new<V: IntoView + 'static>(
        calendar_days: impl Fn() -> CalendarDays + 'static,
        date_view: impl Fn(Date) -> V + 'static,
    ) -> Self {
        let calendar_days = calendar_days;
        let id = ViewId::new();

        let calendar_days = create_memo(move |_| calendar_days());
        let active_date = create_memo(move |_| calendar_days.get().start_date);
        let show_minimum = RwSignal::new(ShowMinimum::default());

        let child = dyn_stack(
            move || {
                let show_minimum = show_minimum.get();
                let calendar_days = calendar_days.get();

                calendar_days
                    .with_show_minimum(show_minimum)
                    .map(move |val| (val, show_minimum))
            },
            |(date, _)| *date,
            move |(date, _show_minimum)| {
                let today = move || date == active_date.get();
                let weekend = move || date.weekday() as u8 >= Weekday::Saturday as u8;
                let other_month = move || {
                    date.month() != active_date.get().month()
                        || date.year() != active_date.get().year()
                };
                date_view(date)
                    .into_any()
                    .class(DateClass)
                    .class_if(other_month, OtherMonth)
                    .class_if(weekend, Weekend)
                    .class_if(today, Today)
            },
        )
        .style(|s| {
            s.grid()
                .grid_template_columns(vec![repeat(GridTrackRepetition::Count(7), vec![fr(1.)])])
                .grid_auto_rows(vec![minmax(min_content(), auto())])
                .size_full()
        })
        .debug_name("Date Picker Grid")
        .class(DateGridClass)
        .into_any();

        id.add_child(child);

        Self {
            id,
            show_minimum,
            style: Default::default(),
        }
        .class(DatePickerClass)
    }

    pub fn basic(calendar_days: impl Fn() -> CalendarDays + 'static) -> Self {
        Self::new(calendar_days, CalendarDay::basic)
    }

    pub fn calendar_style(
        self,
        cal_style: impl Fn(CalendarCustomStyle) -> CalendarCustomStyle + 'static,
    ) -> Self {
        self.custom_style(cal_style)
    }
}

#[derive(Default, Clone, Debug)]
pub struct CalendarCustomStyle(Style);
impl From<CalendarCustomStyle> for Style {
    fn from(custom: CalendarCustomStyle) -> Self {
        custom.0
    }
}
impl CustomStylable<CalendarCustomStyle> for Calendar {
    type DV = Self;
}

impl CalendarCustomStyle {
    pub fn new() -> Self {
        Self(Style::new())
    }

    pub fn show_minimum(mut self, show_minimum: ShowMinimum) -> Self {
        self = Self(self.0.set(ShowMin, show_minimum));
        self
    }
}
