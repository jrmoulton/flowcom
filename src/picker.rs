use std::{any::Any, cell::RefCell, hash::Hash, rc::Rc, sync::Arc, time::Duration};

use floem::{
    action::exec_after,
    ext_event::create_signal_from_channel,
    id::Id,
    keyboard::{Key, ModifiersState, NamedKey},
    kurbo::Rect,
    reactive::{create_effect, RwSignal, Trigger},
    style_class,
    unit::PxPctAuto,
    view::{AnyView, View, ViewData, Widget},
    views::{scroll, v_stack, Decorators, VirtualDirection, VirtualItemSize, VirtualVector},
    widgets::{text_input, virtual_list, ListClass},
};
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Nucleo,
};

#[derive(Clone)]
pub struct NewSnapshot<T: Send + Sync + Clone + 'static>(Rc<RefCell<nucleo::Nucleo<T>>>);
impl<T: Send + Sync + Clone + 'static> VirtualVector<T> for NewSnapshot<T> {
    fn total_len(&self) -> usize {
        let val = self.0.borrow_mut();
        val.snapshot().matched_item_count() as usize
    }

    fn slice(&mut self, range: std::ops::Range<usize>) -> impl Iterator<Item = T> {
        let val = self.0.borrow_mut();
        val.snapshot()
            .matched_items(range.start as u32..range.end as u32)
            .map(|data| data.data.clone())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

pub enum InputLocation {
    Top,
    Bottom,
}

pub fn picker_with_input<T, VF, KF, K>(
    items: impl Fn() -> im::Vector<T> + 'static,
    view_fn: VF,
    key_fn: KF,
    // input_location: InputLocation,
) -> impl View
where
    T: Send + Sync + Clone + ToString + 'static,
    VF: Fn(T) -> AnyView + 'static,
    KF: Fn(&T) -> K + 'static,
    K: Eq + Hash + 'static,
{
    let input_sig = RwSignal::new(String::new());
    let inner_size = RwSignal::new(Rect::ZERO);

    v_stack((
        scroll(
            picker(items, view_fn, key_fn)
                .style(|s| {
                    s.class(ListClass, |s| s.flex_col().width_full())
                        .margin_bottom(0)
                        .width_full()
                })
                .on_resize(move |rect| inner_size.set(rect))
                .update_filter(move || input_sig.get()),
        )
        .style(move |s| {
            s.width_full()
                .margin_top(PxPctAuto::Auto)
                .min_height(0)
                .flex_grow(1.)
                .flex_basis(0.)
                .max_height(inner_size.with(|val| val.height()))
        }),
        text_input(input_sig).style(|s| s.width_full()).on_key_down(
            Key::Named(NamedKey::ArrowUp),
            ModifiersState::empty(),
            |_| {},
        ),
    ))
    .style(|s| s.size_full())
}

style_class!(pub FuzzyPickerClass);

enum PickerUpdate {
    Filter(String),
    NewInfo,
    // type erased but will always be im::Vector<T>
    NewItems(Box<dyn Any>),
}

#[derive(Clone)]
pub enum ResultOrdering {
    TopToBottom,
    BottomToTop,
}

pub struct FuzzyPicker<T: Sync + Send + Clone + 'static> {
    data: ViewData,
    picker: Rc<RefCell<Nucleo<T>>>,
    child: Box<dyn Widget>,
    prev_filter: String,
    update: Trigger,
}

pub fn picker<T, VF, KF, K>(
    items: impl Fn() -> im::Vector<T> + 'static,
    view_fn: VF,
    key_fn: KF,
) -> FuzzyPicker<T>
where
    T: Send + Sync + Clone + ToString + 'static,
    VF: Fn(T) -> AnyView + 'static,
    KF: Fn(&T) -> K + 'static,
    K: Eq + Hash + 'static,
{
    let id = Id::next();

    let (update_tracker_tx, update_tracker_rx) = crossbeam_channel::bounded(2);
    let update_tracker = create_signal_from_channel(update_tracker_rx);

    create_effect(move |_| {
        if update_tracker.get().is_some() {
            id.update_state(PickerUpdate::NewInfo)
        }
    });

    let picker = Rc::new(RefCell::new(Nucleo::new(
        nucleo::Config::DEFAULT,
        Arc::new(move || {
            update_tracker_tx.send(()).unwrap();
        }),
        None,
        1,
    )));

    create_effect(move |_| {
        let items = items();
        id.update_state(PickerUpdate::NewItems(Box::new(items)));
    });

    let new_snap = NewSnapshot(picker.clone());

    let update = Trigger::new();
    let child = virtual_list(
        VirtualDirection::Vertical,
        VirtualItemSize::Fixed(Box::new(|| 17.475)), // TODO: This needs to be made configurable
        move || {
            update.track();
            new_snap.clone()
        },
        move |vals| key_fn(vals),
        view_fn,
    )
    .build();
    FuzzyPicker {
        data: ViewData::new(id),
        picker,
        child,
        prev_filter: String::new(),
        update,
    }
    .class(FuzzyPickerClass)
}

impl<T: Send + Sync + Clone + ToString + 'static> View for FuzzyPicker<T> {
    fn view_data(&self) -> &ViewData {
        &self.data
    }

    fn view_data_mut(&mut self) -> &mut ViewData {
        &mut self.data
    }

    fn build(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl<T: Send + Sync + Clone + ToString + 'static> Widget for FuzzyPicker<T> {
    fn view_data(&self) -> &ViewData {
        &self.data
    }

    fn view_data_mut(&mut self) -> &mut ViewData {
        &mut self.data
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
        if let Ok(state) = state.downcast::<PickerUpdate>() {
            match *state {
                PickerUpdate::Filter(filter) => {
                    self.picker.borrow_mut().pattern.reparse(
                        0,
                        &filter,
                        CaseMatching::Smart,
                        Normalization::Smart,
                        filter.starts_with(&self.prev_filter),
                    );
                    self.id().update_state(PickerUpdate::NewInfo);
                }
                PickerUpdate::NewInfo => {
                    if self.picker.borrow_mut().tick(0).running {
                        let id = self.id();
                        exec_after(Duration::from_millis(3), move |_| {
                            id.update_state(PickerUpdate::NewInfo)
                        });

                        return;
                    }
                    self.update.notify();
                }
                PickerUpdate::NewItems(items) => {
                    if let Ok(items) = items.downcast::<im::Vector<T>>() {
                        self.picker.borrow_mut().restart(false);
                        let injector = self.picker.borrow_mut().injector();

                        for item in *items {
                            injector.push(item.clone(), |cols| cols[0] = item.to_string().into());
                        }
                    }
                }
            }
        }
    }
}

impl<T: Send + Sync + Clone + 'static> FuzzyPicker<T> {
    pub fn on_accept(self) -> Self {
        self
    }

    pub fn on_select() {}

    pub fn update_filter(self, filter: impl Fn() -> String + 'static) -> Self {
        let id = self.data.id();
        create_effect(move |_| {
            let filter = filter();
            id.update_state(PickerUpdate::Filter(filter));
        });
        self
    }
}
