//! Demonstrates use of the Variable Item List module.

#![no_main]
#![no_std]
#![feature(box_into_inner)]

extern crate alloc;
extern crate flipperzero_alloc;
extern crate flipperzero_rt;

use alloc::boxed::Box;
#[cfg(miri)]
use alloc::sync::Arc;
use core::ffi::CStr;
use core::sync::atomic::{AtomicI8, Ordering};
use flipperzero::format;
#[cfg(miri)]
use flipperzero::gui::variable_item_list::{
    self, UniqueCallbackForEachItem, VariableItemListBoundToViewDispatcher,
};
use flipperzero::gui::variable_item_list::{
    Callback, OnCurrentValueTextChangedCallbacks, VariableItem, VariableItemList,
};
use flipperzero::gui::view_dispatcher::{DontBind, ViewDispatcherInner};
use flipperzero::gui::{
    Gui,
    variable_item_list::{
        Callback, MutexGuardedVariableItemType, OnCurrentValueTextChangedCallbacks, VariableItem,
        VariableItemList,
    },
    view_dispatcher::{
        DontBind, StopDispatcher, ViewDispatcher, ViewDispatcherCallbacks, ViewDispatcherInner,
        ViewDispatcherType,
    },
};
use flipperzero::{format, prelude::FuriString};
use flipperzero_rt::{entry, manifest};

manifest!(name = "Rust Variable Item List example");
entry!(main);

#[cfg(miri)]
unsafe extern "Rust" {
    pub fn miri_thread_spawn(t: extern "Rust" fn(*mut ()), data: *mut ()) -> usize;
    pub fn miri_thread_join(thread_id: usize) -> bool;
    pub fn miri_set_thread_name(thread_id: usize, name: *const u8) -> bool;
    pub safe fn miri_write_to_stdout(bytes: &[u8]);
}

#[cfg(not(miri))]
pub fn miri_write_to_stdout(bytes: &[u8]) {}

struct State {}

impl ViewDispatcherCallbacks for State {
    type BindCustom = DontBind;
    type BindTick = DontBind;

    fn on_navigation<T>(&self, _view_dispatcher: &ViewDispatcherInner<T>) -> StopDispatcher
    where
        T: ViewDispatcherCallbacks,
    {
        StopDispatcher::Yes
    }
}

struct IncrementGlobalCounterCallback<'a> {
    counter: &'a AtomicI8,
    increment_by: i8,
}

impl Callback<'_> for IncrementGlobalCounterCallback<'_> {
    fn on_click(&self, _item: MutexGuardedVariableItemType<'_, '_>) -> () {
        {
            let msg = alloc::format!("Incrementing by: {}\n", self.increment_by);
            miri_write_to_stdout(msg.as_bytes());
        }

        self.counter.fetch_add(self.increment_by, Ordering::SeqCst);
    }
}

struct IncrementGlobalCounterByVariableCallback<'a> {
    counter: &'a AtomicI8,
    increment_by: &'a AtomicI8,
}

struct ChangeIncrementAmountCallback<'a> {
    number_of_options: u8,
    min_value: i8,
    increment_amount: &'a AtomicI8,
}

impl Callback<'_> for IncrementGlobalCounterByVariableCallback<'_> {
    fn on_click(&self, _item: MutexGuardedVariableItemType<'_, '_>) -> () {
        let val = self.increment_by.load(Ordering::SeqCst);

        {
            let msg = alloc::format!("Incrementing by variable amount: {}\n", val);
            miri_write_to_stdout(msg.as_bytes());
        }

        self.counter.fetch_add(val, Ordering::SeqCst);
    }
}

impl OnCurrentValueTextChangedCallbacks for ChangeIncrementAmountCallback<'_> {
    fn get_new_label(&self, _item: &VariableItem, value: u8) -> flipperzero::prelude::FuriString {
        let val: i8 = (value as i8) + self.min_value;
        self.increment_amount.store(val, Ordering::SeqCst);

        {
            let msg = alloc::format!("Setting variable increment amount to: {}\n", val);
            miri_write_to_stdout(msg.as_bytes());
        }

        format!("{}", val)
    }
}

fn main(_args: Option<&CStr>) -> i32 {
    let gui = Gui::open();

    #[cfg(miri)]
    let miri_gui = {
        let view_port_gui: Arc<flipperzero_sys::Gui> = unsafe { Arc::from_raw(gui.as_ptr()) };
        let miri_gui = view_port_gui.clone();
        let _ = Arc::into_raw(view_port_gui);
        miri_gui
    };

    let mut view_dispatcher = ViewDispatcher::new(State {}, &gui, ViewDispatcherType::Fullscreen);

    let counter = AtomicI8::new(0);

    let mut variable_item_list = VariableItemList::new();
    variable_item_list.push_item_plaintext(c"First Item".into());
    variable_item_list.push_item_with_on_click_callback(
        "Add two".into(),
        IncrementGlobalCounterCallback {
            counter: &counter,
            increment_by: 2,
        },
    );
    variable_item_list.push_item_with_on_click_callback(
        "Add three".into(),
        IncrementGlobalCounterCallback {
            counter: &counter,
            increment_by: 3,
        },
    );
    variable_item_list.push_item_with_on_click_callback(
        "Subtract one".into(),
        IncrementGlobalCounterCallback {
            counter: &counter,
            increment_by: -1,
        },
    );

    let increment_amount = AtomicI8::new(-2);
    let change_counter_callback = IncrementGlobalCounterByVariableCallback {
        counter: &counter,
        increment_by: &increment_amount,
    };

    let number_of_options = 6;

    let modify_increment_callback = ChangeIncrementAmountCallback {
        number_of_options: number_of_options.clone(),
        min_value: -2,
        increment_amount: &increment_amount,
    };

    variable_item_list
        .push_item_with_on_click_callback("Add variable amount".into(), change_counter_callback);
    variable_item_list.push_item_with_options(
        "Variable amount to add".into(),
        6,
        modify_increment_callback,
    );

    let variable_item_list_view =
        variable_item_list.bind_to_view_dispatcher(0, &mut view_dispatcher);

    variable_item_list_view.switch_to_view();

    #[cfg(not(miri))]
    let status = run_until_exit(view_dispatcher);
    #[cfg(miri)]
    let status = run_until_exit_miri(view_dispatcher, variable_item_list_view, miri_gui, &counter);

    status
}

#[cfg(not(miri))]
fn run_until_exit(view_dispatcher: ViewDispatcher<'_, State>) -> i32 {
    view_dispatcher.run();

    0
}

#[cfg(miri)]
struct SendContext<'callback, 'gui> {
    gui: Arc<flipperzero_sys::Gui>,
    counter: &'callback AtomicI8,
    variable_item_list_view: VariableItemListBoundToViewDispatcher<
        'callback,
        'gui,
        State,
        UniqueCallbackForEachItem<'callback>,
    >,
}

#[cfg(miri)]
fn run_until_exit_miri<'callback, 'gui>(
    view_dispatcher: ViewDispatcher<'gui, State>,
    variable_item_list_view: VariableItemListBoundToViewDispatcher<
        'callback,
        'gui,
        State,
        UniqueCallbackForEachItem<'callback>,
    >,
    gui: Arc<flipperzero_sys::Gui>,
    counter: &'callback AtomicI8,
) -> i32 {
    use alloc::sync::Arc;

    let context = SendContext {
        gui,
        counter,
        variable_item_list_view,
    };
    let thread_id = thread_spawn(send_events_for_miri, context);

    let view_dispatcher = view_dispatcher.run();
    miri_write_to_stdout(b"View Dispatcher returned from run\n");

    unsafe { miri_thread_join(thread_id) };

    assert_eq!(counter.load(Ordering::SeqCst), 0);

    0
}

#[cfg(miri)]
fn thread_spawn<F: FnOnce(T), T: Send>(f: F, t: T) -> usize {
    struct ThreadContext<F: FnOnce(T), T>(F, T);

    extern "Rust" fn dispatch_thread<F: FnOnce(T), T>(data: *mut ()) {
        let data = unsafe { Box::from_raw(data.cast()) };
        let ThreadContext(callback, arg): ThreadContext<F, T> = Box::into_inner(data);

        callback(arg)
    }

    let context = Box::new(ThreadContext(f, t));

    let thread_id = {
        // SAFETY: Arc was generated above
        unsafe { miri_thread_spawn(dispatch_thread::<F, T>, Box::into_raw(context).cast()) }
    };

    unsafe { miri_set_thread_name(thread_id, c"miri event sender".as_ptr()) };

    thread_id
}

#[cfg(miri)]
fn send_events_for_miri<'callback, 'gui>(mut context: SendContext<'callback, 'gui>) {
    use flipperzero::input::miri::send;

    let counter = &context.counter;
    let gui = &context.gui;

    send!(Ok event to gui); // do nothing, this is a plaintext item
    assert_eq!(counter.load(Ordering::SeqCst), 0);

    send!(Down event to gui); // move to +2
    send!(Ok event to gui); // add 2
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    send!(Down event to gui); // move to +3
    send!(Down event to gui); // move to -1
    send!(Ok event to gui); // subtract 1
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    send!(Down event to gui); // move to add variable amount item
    send!(Ok event to gui); // add default variable amount, which is -2
    assert_eq!(counter.load(Ordering::SeqCst), -1);

    send!(Down event to gui); // move to the setting for the variable amount to add
    send!(Right event to gui 4 times); // increase to 2
    send!(Up event to gui); // move to add variable amount item
    send!(Ok event to gui); // add variable amount, which is 2
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    send!(Down event to gui); // move to the setting for the variable amount to add
    send!(Left event to gui); // decrease to 1
    send!(Up event to gui); // move to add variable amount item
    send!(Ok event to gui); // add variable amount, which is 1
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    let variable_item_list = &mut context.variable_item_list_view;
    variable_item_list.push_item_with_on_click_callback(
        "New Item".into(),
        IncrementGlobalCounterCallback {
            counter: &counter,
            increment_by: -1,
        },
    );
    send!(Down event to gui 2 times); // move to the new item
    send!(Ok event to gui 2 times); // subtract 1 twice

    send!(Back event to gui); // back event to exit out
}

#[cfg(miri)]
#[unsafe(no_mangle)]
fn miri_start(_argc: isize, _argv: *const *const u8) -> isize {
    // TODO: Is there any benefit to Miri in hooking up the binary arguments to
    // the test runner?
    let ret = main(None);

    ret.try_into().unwrap_or(isize::MAX)
}
