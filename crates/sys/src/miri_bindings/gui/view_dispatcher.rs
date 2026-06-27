extern crate alloc;

use crate::CallbackWithContext;
use crate::lock::{SpinLock, SpinLockGuard};
use crate::miri_bindings::gui::canvas::Canvas;
use crate::miri_bindings::gui::view_port::{ViewPort, view_port_alloc};
use crate::miri_bindings::gui::{GuiLayerDesktop, gui_add_view_port};
use crate::miri_bindings::input::{InputEvent, InputKeyBack, InputTypeLong, InputTypeShort};
use crate::miri_bindings::utils::*;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, btree_map::Entry};
use alloc::sync::Arc;
use core::ffi::c_void;
use core::ptr::NonNull;

#[doc = "< Desktop layer: fullscreen with status bar on top of it. For internal usage."]
pub const ViewDispatcherTypeDesktop: ViewDispatcherType = ViewDispatcherType(0);
#[doc = "< Window layer: with status bar"]
pub const ViewDispatcherTypeWindow: ViewDispatcherType = ViewDispatcherType(1);
#[doc = "< Fullscreen layer: without status bar"]
pub const ViewDispatcherTypeFullscreen: ViewDispatcherType = ViewDispatcherType(2);
#[repr(transparent)]
#[doc = "ViewDispatcher view_port placement"]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ViewDispatcherType(pub core::ffi::c_uchar);

#[derive(Debug)]
enum InputChannel {
    Pending(Arc<InputEvent>),
    Processing,
}

pub struct ViewDispatcherInner {
    view_port: NonNull<ViewPort>,

    pub custom_event_callback: Option<ViewDispatcherCustomEventCallback>,
    pub navigation_event_callback: Option<ViewDispatcherNavigationEventCallback>,
    pub tick_event_callback: Option<ViewDispatcherTickEventCallback>,
    pub context: *mut c_void,

    views: BTreeMap<u32, NonNull<super::View>>,
    current_view: Option<u32>,

    input_channel: Option<InputChannel>,
    event_channel: Option<u32>,
    stop: bool,
}

impl ViewDispatcherInner {
    fn process_input(this: &mut SpinLockGuard<Self>) -> () {
        let Some(mut input_event) = this.input_channel.replace(InputChannel::Processing) else {
            unreachable!(
                "Checked before entering this method that the input_channel was populated, and we're the only thread that can take from it"
            )
        };

        let InputChannel::Pending(mut input_event) = input_event else {
            unreachable!(
                "Checked before entering this method that the input_channel was populated, and we're the only thread that populate the channel with the Processing state"
            )
        };

        let input_event = unsafe { Arc::get_mut_unchecked(&mut input_event) };

        miri_write_to_stdout(b"View dispatcher process input event\n");

        let Some(ref current_view_id) = this.current_view else {
            miri_write_to_stdout(b"View dispatcher attempted to process input event, but there was no current view\n");
            return;
        };

        let current_view = this
            .views
            .get(current_view_id)
            .expect("The existence was checked on insert");
        let current_view = unsafe { current_view.as_ptr() };
        let current_view = unsafe { &mut *current_view };

        this.unlock();
        let is_consumed = current_view.process_input(input_event);
        this.reacquire();

        if is_consumed {
            core::assert_matches!(this.input_channel.take(), Some(InputChannel::Processing));

            return;
        }

        miri_write_to_stdout(b"View dispatcher's current view did not consume the input event\n");

        if input_event.key != InputKeyBack {
            miri_write_to_stdout(b"Input event was not a back event, no further processing\n");
            core::assert_matches!(this.input_channel.take(), Some(InputChannel::Processing));
            return;
        }

        miri_write_to_stdout(b"Input event was a back event...\n");

        if !(input_event.type_ == InputTypeShort || input_event.type_ == InputTypeLong) {
            miri_write_to_stdout(b"but was not the right type\n");
            core::assert_matches!(this.input_channel.take(), Some(InputChannel::Processing));
            return;
        }

        miri_write_to_stdout(b"and the key was released\n");

        let view_to_switch_to = current_view.process_previous();
        match view_to_switch_to {
            super::view::IGNORE => {
                miri_write_to_stdout(b"The current view did not declare a view to switch to, checking dispatcher's navigation event callback\n");

                let Some(navigation_event_callback) = this.navigation_event_callback else {
                    miri_write_to_stdout(b"Dispatcher does not have a navigation event callback\n");
                    return;
                };

                let navigation_event_callback = navigation_event_callback.expect(
                    "ViewDispatcherNavigationEventCallback is only nullable for FFI reasons",
                );
                let should_stop = unsafe { navigation_event_callback(this.context) };

                if should_stop {
                    miri_write_to_stdout(b"Dispatcher wants to stop running\n");
                    this.stop = true;
                } else {
                    miri_write_to_stdout(b"Dispatcher did not react to back event\n");
                }
            }
            _ => {
                miri_write_to_stdout(b"The current view wants to switch to view \"");
                miri_write_to_stdout(&[char::from_digit(view_to_switch_to, 10).unwrap() as u8]);
                miri_write_to_stdout(b"\"\n");

                this.switch_to_view(view_to_switch_to);
            }
        }

        core::assert_matches!(this.input_channel.take(), Some(InputChannel::Processing));
    }

    fn switch_to_view(&mut self, view_id: u32) -> () {
        if self.views.contains_key(&view_id) {
            self.current_view = Some(view_id);
        } else {
            unimplemented!("Attempted to switch to a view with an id that was not found");
        }
    }
}

#[repr(C)]
pub struct ViewDispatcher {
    inner: Arc<SpinLock<ViewDispatcherInner>>,

    gui: Option<Arc<SpinLock<super::GuiInner>>>,
}

impl ViewDispatcher {
    fn run(&self) -> () {
        let inner = Arc::clone(&self.inner);
        loop {
            // miri_write_to_stdout(b"View Dispatcher loop!\n");
            let mut view_dispatcher_guard = inner.lock("view dispatcher event loop");

            if view_dispatcher_guard.input_channel.is_some() {
                ViewDispatcherInner::process_input(&mut view_dispatcher_guard);
            }

            if view_dispatcher_guard.event_channel.is_some() {
                todo!()
            }

            if view_dispatcher_guard.stop {
                miri_write_to_stdout(b"View Dispatcher loop stopped\n");
                break;
            }

            drop(view_dispatcher_guard);
            miri_spin_loop();
        }
    }
}

#[doc = "Prototype for custom event callback"]
pub type ViewDispatcherCustomEventCallback =
    ::core::option::Option<unsafe extern "C" fn(context: *mut c_void, event: u32) -> bool>;
#[doc = "Prototype for navigation event callback"]
pub type ViewDispatcherNavigationEventCallback =
    ::core::option::Option<unsafe extern "C" fn(context: *mut c_void) -> bool>;
#[doc = "Prototype for tick event callback"]
pub type ViewDispatcherTickEventCallback =
    ::core::option::Option<unsafe extern "C" fn(context: *mut c_void)>;

#[doc = "Allocate ViewDispatcher instance\n\n # Returns\n\npointer to ViewDispatcher instance"]
pub unsafe fn view_dispatcher_alloc() -> *mut ViewDispatcher {
    let view_port = unsafe { NonNull::new_unchecked(view_port_alloc()) };

    let view_dispatcher = ViewDispatcher {
        inner: Arc::new(SpinLock::new(
            ViewDispatcherInner {
                view_port,
                custom_event_callback: None,
                navigation_event_callback: None,
                tick_event_callback: None,
                views: BTreeMap::new(),
                current_view: None,
                context: core::ptr::null_mut(),

                input_channel: None,
                event_channel: None,
                stop: false,
            },
            Some("view dispatcher inner"),
        )),
        gui: None,
    };

    {
        pub unsafe extern "C" fn view_port_dispatch_draw(
            canvas: *mut Canvas,
            context: *mut c_void,
        ) {
            miri_write_to_stdout(b"View dispatcher's view port dispatching draw\n");
            let view_dispatcher =
                unsafe { Arc::from_raw(context as *const SpinLock<ViewDispatcherInner>) };

            {
                let mut view_dispatcher_guard = view_dispatcher.lock("dispatch draw");

                let Some(current_view_id) = view_dispatcher_guard.current_view else {
                    miri_write_to_stdout(b"View dispatcher attempted to process input event, but there was no current view\n");
                    return;
                };

                let current_view = view_dispatcher_guard
                    .views
                    .get_mut(&current_view_id)
                    .expect("The existence was checked on insert");

                unsafe { current_view.as_mut() }.draw(canvas);
            }

            let _ = Arc::into_raw(view_dispatcher);
        }

        pub unsafe extern "C" fn view_port_queue_input_event(
            input_event: *mut InputEvent,
            context: *mut c_void,
        ) {
            // NOTE: we are holding the GUI lock here

            unsafe { Arc::increment_strong_count(input_event) };
            let input_event = unsafe { Arc::from_raw(input_event) };

            miri_write_to_stdout(b"View dispatcher's view port queuing input event\n");
            let view_dispatcher =
                unsafe { Arc::from_raw(context as *const SpinLock<ViewDispatcherInner>) };

            {
                let mut view_dispatcher_guard = view_dispatcher.lock("dispatch input");

                let old_input_event = view_dispatcher_guard
                    .input_channel
                    .replace(InputChannel::Pending(input_event));
                debug_assert!(old_input_event.is_none());
            }

            // NOTE: we block here until the input channel has been taken, because otherwise,
            // execution will return to the caller (the GUI inner's `process_input`) too soon, (which
            // would in turn release the lock on the GUI inner's `send_input_event` method) which
            // might lead to another input event being sent while this one is processed, which
            // could fail the debug_assert above.
            //
            // OPTIMISATION: we unlock the dispatcher here to allow the service thread to `take`
            // the input event we just inserted. there's no point doing that if we're not going to
            // yield here to allow that other thread to run.
            //
            // even without this, we'll yield in the loop below anyway. additionally, miri is
            // probably able to randomly switch threads, and so we might get lucky any not need to
            // loop anyway
            miri_spin_loop();

            // spin until the other thread takes the input out of the channel
            loop {
                // NOTE: this can suffer from the ABBA problem, as we're holding the GUI lock here,
                // and waiting on the view_dispatcher lock, while the view_dispatcher run thread
                // takes the view_dispatcher lock first, and then may invoke callbacks that could
                // try and take the GUI lock
                //
                // this isn't a problem in the flipper codebase because their input channel can
                // store more than one input event, and so they don't need to check that the input
                // event is consumed before queuing another
                let mut view_dispatcher_guard =
                    view_dispatcher.lock("check for input event consumed");
                if view_dispatcher_guard.input_channel.is_none() {
                    miri_write_to_stdout(b"View dispatcher consumed input event\n");
                    break;
                }

                miri_spin_loop();
            }

            let _ = Arc::into_raw(view_dispatcher);
        }

        let context = Arc::into_raw(view_dispatcher.inner.clone());
        let context = context.cast::<c_void>().cast_mut();

        let mut view_dispatcher = view_dispatcher.inner.lock("init");
        let mut view_port = (unsafe { view_dispatcher.view_port.as_mut() })
            .inner
            .lock("init");

        view_port.draw_callback = Some(CallbackWithContext {
            callback: Some(view_port_dispatch_draw),
            context,
        });
        view_port.input_callback = Some(CallbackWithContext {
            callback: Some(view_port_queue_input_event),
            context,
        });
    }

    debug_assert_eq!(
        Arc::strong_count(&view_dispatcher.inner),
        2,
        "[view_dispatcher_alloc, view_dispatcher's view_port callback context]"
    );

    Box::into_raw(Box::new(view_dispatcher))
}

#[doc = "Free ViewDispatcher instance\n\n All added views MUST be removed using view_dispatcher_remove_view()\n before calling this function.\n\n # Arguments\n\n* `view_dispatcher` - pointer to ViewDispatcher"]
pub unsafe fn view_dispatcher_free(view_dispatcher: *mut ViewDispatcher) {
    let view_dispatcher = unsafe { Box::from_raw(view_dispatcher) };
    let view_dispatcher = view_dispatcher.inner.lock("free");
    unsafe { super::view_port_free(view_dispatcher.view_port.as_ptr()) };
}

#[doc = "Enable queue support\n\n > **Deprecated** Do NOT use in new code and remove all calls to it from existing code.\n The queue support is now always enabled during construction. If no queue support\n is required, consider using ViewHolder instead.\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance"]
pub unsafe fn view_dispatcher_enable_queue(view_dispatcher: *mut ViewDispatcher) {
    unimplemented!("This method is deprecated")
}
#[doc = "Send custom event\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance\n * `event` (direction in) - The event"]
pub unsafe fn view_dispatcher_send_custom_event(view_dispatcher: *mut ViewDispatcher, event: u32) {
    todo!()
}
#[doc = "Set custom event handler\n\n Called on Custom Event, if it is not consumed by view\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance\n * `callback` - ViewDispatcherCustomEventCallback instance"]
pub unsafe fn view_dispatcher_set_custom_event_callback(
    view_dispatcher: *mut ViewDispatcher,
    callback: ViewDispatcherCustomEventCallback,
) {
    let view_dispatcher = unsafe { &*view_dispatcher };
    let mut view_dispatcher = view_dispatcher.inner.lock("set event callback");
    view_dispatcher.custom_event_callback = Some(callback);
}
#[doc = "Set navigation event handler\n\n Called on Input Short Back Event, if it is not consumed by view\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance\n * `callback` - ViewDispatcherNavigationEventCallback instance"]
pub unsafe fn view_dispatcher_set_navigation_event_callback(
    view_dispatcher: *mut ViewDispatcher,
    callback: ViewDispatcherNavigationEventCallback,
) {
    let view_dispatcher = unsafe { &*view_dispatcher };
    let mut view_dispatcher = view_dispatcher.inner.lock("set nav event callback");
    view_dispatcher.navigation_event_callback = Some(callback);
}
#[doc = "Set tick event handler\n\n Requires the event loop to be owned by the view dispatcher, i.e.\n it should have been instantiated with `view_dispatcher_alloc`, not\n `view_dispatcher_alloc_ex`.\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance\n * `callback` - ViewDispatcherTickEventCallback\n * `tick_period` - callback call period"]
pub unsafe fn view_dispatcher_set_tick_event_callback(
    view_dispatcher: *mut ViewDispatcher,
    callback: ViewDispatcherTickEventCallback,
    tick_period: u32,
) {
    todo!()
}
#[doc = "Set event callback context\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance\n * `context` - pointer to context"]
pub unsafe fn view_dispatcher_set_event_callback_context(
    view_dispatcher: *mut ViewDispatcher,
    context: *mut c_void,
) {
    let view_dispatcher = unsafe { &mut *view_dispatcher };
    let mut view_dispatcher = view_dispatcher.inner.lock("set context");
    view_dispatcher.context = context;
}
#[doc = "Run ViewDispatcher\n\n This function will start the event loop and block until view_dispatcher_stop() is called\n or the current thread receives a FuriSignalExit signal.\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance"]
pub unsafe fn view_dispatcher_run(view_dispatcher: *mut ViewDispatcher) {
    let view_dispatcher = unsafe { &*view_dispatcher };
    view_dispatcher.run()
}
#[doc = "Stop ViewDispatcher\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance"]
pub unsafe fn view_dispatcher_stop(view_dispatcher: *mut ViewDispatcher) {
    todo!()
}

#[doc = "Add view to ViewDispatcher\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance\n * `view_id` - View id to register\n * `view` - View instance"]
pub unsafe fn view_dispatcher_add_view(
    view_dispatcher: *mut ViewDispatcher,
    view_id: u32,
    view: *mut super::View,
) {
    let view_dispatcher: &ViewDispatcher = unsafe { &*view_dispatcher };

    miri_write_to_stdout(b"Attempting to take GUI lock\n");
    let guard = view_dispatcher.gui.as_deref().map(|l| l.lock("add view"));

    miri_write_to_stdout(b"Attempting to take view dispatcher lock\n");
    let mut view_dispatcher = view_dispatcher.inner.lock("add view");
    let Entry::Vacant(entry) = view_dispatcher.views.entry(view_id) else {
        panic!("The view_id is already in use");
    };

    let view = unsafe { NonNull::new_unchecked(view) };
    let _ = entry.insert(view);

    miri_write_to_stdout(b"Successfully added view\n");
}

#[doc = "Remove view from ViewDispatcher\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance\n * `view_id` - View id to remove"]
pub unsafe fn view_dispatcher_remove_view(view_dispatcher: *mut ViewDispatcher, view_id: u32) {
    let view_dispatcher: &ViewDispatcher = unsafe { &*view_dispatcher };

    miri_write_to_stdout(b"Attempting to take gui lock in order to remove view from dispatcher\n");
    let guard = view_dispatcher
        .gui
        .as_deref()
        .map(|l| l.lock("remove view"));

    miri_write_to_stdout(
        b"Attempting to take view dispatcher lock in order to remove view from dispatcher\n",
    );
    let mut view_dispatcher = view_dispatcher.inner.lock("remove view");

    match view_dispatcher.views.remove(&view_id) {
        Some(_) => miri_write_to_stdout(b"Successfully removed view\n"),
        None => miri_write_to_stdout(b"Could not remove view at id\n"),
    }
}

#[doc = "Switch to View\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance\n * `view_id` - View id to register\n switching may be delayed till input events complementarity\n reached"]
pub unsafe fn view_dispatcher_switch_to_view(view_dispatcher: *mut ViewDispatcher, view_id: u32) {
    let view_dispatcher: &ViewDispatcher = unsafe { &*view_dispatcher };

    miri_write_to_stdout(b"Attempting to take view dispatcher lock\n");
    let mut view_dispatcher = view_dispatcher.inner.lock("switch to view");

    if view_dispatcher.views.contains_key(&view_id) {
        view_dispatcher.current_view = Some(view_id);
    } else {
        unimplemented!("Attempted to switch to a view with an id that was not found");
    }
}
#[doc = "Send ViewPort of this ViewDispatcher instance to front\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance"]
pub unsafe fn view_dispatcher_send_to_front(view_dispatcher: *mut ViewDispatcher) {
    todo!()
}
#[doc = "Send ViewPort of this ViewDispatcher instance to back\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance"]
pub unsafe fn view_dispatcher_send_to_back(view_dispatcher: *mut ViewDispatcher) {
    todo!()
}
#[doc = "Attach ViewDispatcher to GUI\n\n # Arguments\n\n* `view_dispatcher` - ViewDispatcher instance\n * `gui` - GUI instance to attach to\n * `type` (direction in) - The type"]
pub unsafe fn view_dispatcher_attach_to_gui(
    view_dispatcher: *mut ViewDispatcher,
    gui: *mut super::Gui,
    type_: ViewDispatcherType,
) {
    let view_dispatcher: &mut ViewDispatcher = unsafe { &mut *view_dispatcher };

    debug_assert_eq!(
        Arc::strong_count(&view_dispatcher.inner),
        2,
        "[dispatcher::new, dispatcher callback context]"
    );

    let main_gui = unsafe { Arc::from_raw(gui) };
    view_dispatcher.gui = Some(main_gui.clone());
    let _ = Arc::into_raw(main_gui);

    let view_port = {
        let view_dispatcher = view_dispatcher.inner.lock("attach dispatcher to gui");
        view_dispatcher.view_port.as_ptr()
    };

    unsafe { gui_add_view_port(gui, view_port, GuiLayerDesktop) };
}
