use chrono::Local;
use floem::{
    event::{Event, EventListener},
    keyboard::{Key, NamedKey},
    peniko::Color,
    taffy::AlignContent,
    unit::UnitExt,
    view::{View as _, Widget},
    views::{container, Decorators as _},
    widgets::ListItemClass,
};
use flowcom::datepicker::{self, DateClass, DateContainerClass};

pub fn main() {
    let app_view = container(
        datepicker::datepicker(
            move || Local::now().date_naive(),
            move || Local::now().date_naive(),
        )
        .style(|s| {
            s.width(400)
                .aspect_ratio(1.2)
                .font_size(15.)
                .class(DateContainerClass, |s| {
                    s.outline(0.5)
                        // .outline_color(Color::BLACK)
                        .class(DateClass, |s| {
                            s.border_radius(100.pct())
                                .width(75.pct())
                                // .focus(|s| s.background(Color::GREEN).color(Color::WHITE))
                                .selected(|s| s.background(Color::GREEN).color(Color::WHITE))
                        })
                })
        }),
    )
    .style(|s| s.size_full().justify_center().items_center());
    let id = app_view.id();
    // set global keyboard shortcuts. They are global because they are handled on
    // the main app view
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
