use floem::{
    event::{Event, EventListener},
    keyboard::{Key, NamedKey},
    peniko::Color,
    reactive::RwSignal,
    unit::UnitExt,
    views::{scroll, stack, text_input, toggle_button, Decorators},
    View as _,
};
use flowcom::calendar::{self, CalendarCustomStyle, CalendarDays, DatePickerClass};
use jiff::Zoned;

pub fn main() {
    let black = Color::parse("#2D2D2D").unwrap();
    let not_focused1 = Color::parse("#7B7B7B").unwrap();
    let not_focused2 = Color::parse("#B6B6B6").unwrap();
    let weekend_background = Color::parse("#EFEFEF").unwrap();
    let border_color = Color::parse("#E5E5E5").unwrap();

    let calendar_days = RwSignal::new(CalendarDays {
        start_date: Zoned::now().date(),
        end_date: Zoned::now().date(),
    });

    let extra_days = RwSignal::new(String::new());

    let show_other_dates = RwSignal::new(false);
    let window_aspect_ratio = RwSignal::new(1.0);

    let date_picker = calendar::Calendar::basic(move || {
        calendar_days.get() + extra_days.get().parse::<i32>().unwrap_or_default()
    })
    .style(move |s| {
        use calendar::*;
        s.class(DateClass, |s| {
            s.outline(0.5)
                .outline_color(border_color)
                .items_start()
                .justify_end()
                .aspect_ratio(window_aspect_ratio.get())
                .padding_top(5.pct())
                .padding_right(8.pct())
        })
        .class(calendar::OtherMonth, move |s| {
            s.color(not_focused2)
                .apply_if(!show_other_dates.get(), |s| {
                    s.color(Color::TRANSPARENT)
                        .outline_color(Color::TRANSPARENT)
                })
        })
        .class(calendar::Weekend, move |s| {
            s.background(weekend_background).color(not_focused1)
        })
        .class(DateGridClass, |s| s.gap(5))
        .width_full()
    });

    let toggle_button = toggle_button(move || show_other_dates.get())
        .on_toggle(move |state| show_other_dates.set(state));

    let num_days_input = text_input(extra_days);

    let app_view = stack((
        scroll(date_picker).scroll_style(|s| s.shrink_to_fit()),
        ("Show all dates", toggle_button).style(|s| s.gap(10).items_center()),
        ("Number of extra days", num_days_input).style(|s| s.gap(10).items_center()),
    ))
    .style(move |s| {
        s.size_full()
            .justify_center()
            .items_center()
            .padding(15.)
            .gap(10)
            .color(black)
            .flex_col()
            .class(DatePickerClass, |s| {
                s.apply_custom(CalendarCustomStyle::new().show_minimum(calendar::ShowMinimum::Week))
            })
    })
    .on_resize(move |rect| window_aspect_ratio.set(1. / rect.aspect_ratio() as f32));
    let id = app_view.id();
    let app_view = app_view.on_event_stop(EventListener::KeyUp, move |e| {
        if let Event::KeyUp(e) = e {
            // F11 for the inspector
            if e.key.logical_key == Key::Named(NamedKey::F11) {
                id.inspect();
            }
        }
    });

    floem::launch(move || app_view);
}
