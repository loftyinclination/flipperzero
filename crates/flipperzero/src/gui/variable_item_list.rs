//! A list, where each item may have selectable options, and a corresponding label. See any
//! settings page.

use crate::furi::string::FuriString;
use crate::furi::sync::{FuriMutex, Mutex};
use crate::gui::view::View;
use crate::gui::view_dispatcher::{ViewDispatcher, ViewDispatcherCallbacks, ViewDispatcherView};
use alloc::sync::Arc;
use alloc::{boxed::Box, vec::Vec};
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};
use core::{
    ffi::{CStr, c_char, c_void},
    ptr::{self, NonNull},
};
use flipperzero_sys as sys;
use lock_api::MappedMutexGuard;

/// The Item List.
pub struct VariableItemList<'a, T> {
    inner: VariableItemListInner,
    context: Arc<CallbackContext<'a, T>>,
}

/// A safe wrapper around the [sys::VariableItemList].
struct VariableItemListInner(NonNull<sys::VariableItemList>);

impl VariableItemListInner {
    fn as_ptr(&self) -> *mut sys::VariableItemList {
        self.0.as_ptr()
    }
}

unsafe impl Send for VariableItemListInner {}

/// An item in the list.
pub struct VariableItem {
    list_index: usize,
    inner: NonNull<sys::VariableItem>,
    parent: NonNull<sys::VariableItemList>,
}

impl Debug for VariableItem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VariableItem")
            .field("list_index", &self.list_index)
            .finish()
    }
}

unsafe impl Send for VariableItem {}

struct CallbackContext<'a, T: 'a> {
    callback: Mutex<T>,
    strings: Mutex<Vec<FuriString>>,
    items: Mutex<Vec<VariableItemType<'a>>>,
}

impl<T: Debug> Debug for CallbackContext<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CallbackContext")
            .field("callback", &self.callback)
            .field("items", &self.items)
            .finish()
    }
}

impl<'a, T> CallbackContext<'a, T> {
    fn get_item_at_index(
        &self,
        index: u32,
    ) -> lock_api::MappedMutexGuard<'_, crate::furi::sync::FuriMutex, VariableItemType<'a>> {
        let Ok(res) = lock_api::MutexGuard::try_map(self.items.lock(), |context| {
            context.get_mut(index as usize)
        }) else {
            unreachable!(
                "List index was gotten from inserting, so there should always be an item at the index"
            )
        };

        res
    }
}

pub struct VariableItemRef<'a, T> {
    context: Arc<CallbackContext<'a, T>>,
    list_index: usize,
}

impl<T> ufmt::uDebug for VariableItemRef<'_, T> {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        f.debug_struct("VariableItemRef")?
            .field("index", &self.list_index)?
            .field("label", self.get_label_mut().deref())?
            .finish()
    }
}

impl<'a, T> VariableItemRef<'a, T> {
    /// Locks the item list, and returns a mutable reference to the reference's associated
    /// item-type.
    pub fn get_mut(
        &self,
    ) -> lock_api::MappedMutexGuard<'_, crate::furi::sync::FuriMutex, VariableItemType<'a>> {
        lock_api::MutexGuard::try_map(self.context.items.lock(), |context| {
            context.get_mut(self.list_index)
        })
        .expect(
            "List index was gotten from inserting, so there should always be an item at the index",
        )
    }

    /// Locks the string list, and returns a mutable reference to the reference's label.
    ///
    /// Changes to this label will be applied the next time that the variable item list is redrawn
    /// (perhaps by [VariableItemValueCallbacksContext::set_number_of_options],
    /// [VariableItemValueCallbacksContext::set_currently_selected_value], or
    /// [VariableItemValueCallbacksContext::override_value_label]).
    pub fn get_label_mut(
        &self,
    ) -> lock_api::MappedMutexGuard<'_, crate::furi::sync::FuriMutex, FuriString> {
        lock_api::MutexGuard::try_map(self.context.strings.lock(), |context| {
            context.get_mut(self.list_index)
        })
        .expect(
            "List index was gotten from inserting, so there should always be an item at the index",
        )
    }
}

#[derive(Debug)]
pub enum VariableItemType<'a> {
    Plain(VariableItem),
    // NOTE: we need to wrap the callbacks context in a smart pointer here, so that we can point to
    // it from the FFI callback
    WithValues(Box<VariableItemValueCallbacksContext<'a>>),
}

pub struct UniqueCallbackForEachItem<'a>(Vec<(usize, Box<dyn Callback<'a> + 'a>)>);

impl<'a> CallbackContext<'a, UniqueCallbackForEachItem<'a>> {
    fn try_get_callback_for_item_at_index(
        &self,
        index: u32,
    ) -> Option<
        lock_api::MappedMutexGuard<'_, crate::furi::sync::FuriMutex, Box<dyn Callback<'a> + 'a>>,
    > {
        lock_api::MutexGuard::try_map(self.callback.lock(), |context| {
            context
                .0
                .iter_mut()
                .find_map(|(item_id, callback)| (*item_id == index as usize).then_some(callback))
        })
        .ok()
    }
}

pub struct VariableItemValueCallbacksContext<'a> {
    label_ptr: *const c_char,
    callbacks: Box<dyn OnCurrentValueTextChangedCallbacks + 'a>,
    value_label: FuriString,
    item: VariableItem,
    number_of_options: u8,
}

unsafe impl Send for VariableItemValueCallbacksContext<'_> {}

impl Debug for VariableItemValueCallbacksContext<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VariableItemValueCallbacksContext")
            .field("value_label", &self.value_label)
            .field("list_index", &self.item.list_index)
            .finish()
    }
}

pub type MutexGuardedVariableItemType<'guard, 'callbacks> =
    MappedMutexGuard<'guard, FuriMutex, VariableItemType<'callbacks>>;

pub trait Callback<'callbacks>: Send {
    /// Called on a (short) Ok input event.
    fn on_click<'guard>(&'guard self, item: MutexGuardedVariableItemType<'guard, 'callbacks>)
    -> ();
}

pub trait OnCurrentValueTextChangedCallbacks: Send {
    /// Called in response to the user pressing changing this [Variable Item's](VariableItem)
    /// selected value (by pressing left or right while this item is selected in the list). Should
    /// return the new value for the option.
    ///
    /// Note that this is not invoked when the value is changed via methods on
    /// [VariableItemValueCallbacksContext].
    fn get_new_label(&self, item: &VariableItem, value: u8) -> FuriString;

    /// An optional callback that allows for changes to the value to have further impact, in
    /// addition to just having the label change.
    fn react_to_change(&self) -> () {}
}

impl<'callbacks> VariableItemList<'callbacks, UniqueCallbackForEachItem<'callbacks>> {
    /// Creates a new variable item list, for which each item may perform a different action when
    /// clicked.
    pub fn new() -> Self {
        let inner = {
            let variable_item_list = unsafe { sys::variable_item_list_alloc() };
            VariableItemListInner(unsafe { NonNull::new_unchecked(variable_item_list) })
        };

        unsafe extern "C" fn dispatch_callback<'callbacks>(context: *mut c_void, index: u32) -> () {
            unsafe { Arc::increment_strong_count(context) };

            let context: Arc<CallbackContext<UniqueCallbackForEachItem>> =
                unsafe { Arc::from_raw(context.cast()) };

            let Some(callback_for_item) = context.try_get_callback_for_item_at_index(index) else {
                return;
            };

            let item = context.get_item_at_index(index);

            callback_for_item.on_click(item);
        }

        let callback_context = CallbackContext {
            callback: Mutex::new(UniqueCallbackForEachItem(Vec::new())),
            strings: Mutex::new(Vec::new()),
            items: Default::default(),
        };

        let res = Self {
            inner,
            context: Arc::new(callback_context),
        };

        unsafe {
            sys::variable_item_list_set_enter_callback(
                res.inner.as_ptr(),
                Some(dispatch_callback),
                Arc::as_ptr(&res.context).cast_mut().cast(),
            );
        };

        res
    }

    // NOTE: Label must be owned here; the pointer must be valid for as long as the item exists.
    // Unless we want to accept a CStr and return something with a lifetime, and require the user
    // to keep track of that, this is the best we've got.
    /// Push a plaintext item to the end of the variable item list.
    pub fn push_item_plaintext(
        &mut self,
        label: FuriString,
    ) -> VariableItemRef<'callbacks, UniqueCallbackForEachItem<'callbacks>> {
        let mut items_guard = self.context.items.lock();

        let variable_item = unsafe {
            sys::variable_item_list_add(self.as_raw(), label.as_c_ptr(), 0, None, ptr::null_mut())
        };

        let inner = unsafe { NonNull::new_unchecked(variable_item) };
        let list_index = items_guard.len();
        let item = VariableItem {
            inner,
            list_index,
            parent: self.inner.0,
        };

        items_guard.push(VariableItemType::Plain(item));
        self.context.strings.lock().push(label);

        drop(items_guard);

        VariableItemRef {
            context: self.context.clone(),
            list_index,
        }
    }

    /// Push an item to the end of the variable item list that, when clicked on, invokes a
    /// callback.
    pub fn push_item_with_on_click_callback<C: Callback<'callbacks> + 'callbacks>(
        &mut self,
        label: FuriString,
        callback: C,
    ) -> VariableItemRef<'callbacks, UniqueCallbackForEachItem<'callbacks>> {
        let mut items_guard = self.context.items.lock();

        let variable_item = unsafe {
            sys::variable_item_list_add(self.as_raw(), label.as_c_ptr(), 0, None, ptr::null_mut())
        };

        let inner = unsafe { NonNull::new_unchecked(variable_item) };
        let list_index = items_guard.len();
        let item = VariableItem {
            inner,
            list_index,
            parent: self.inner.0,
        };

        items_guard.push(VariableItemType::Plain(item));
        self.context.strings.lock().push(label);

        drop(items_guard);

        self.context
            .callback
            .lock()
            .0
            .push((list_index, Box::new(callback)));

        VariableItemRef {
            context: self.context.clone(),
            list_index,
        }
    }

    /// Push an item to the end of the variable item list. The item will have a number of options
    /// which can be selected.
    pub fn push_item_with_options<C: OnCurrentValueTextChangedCallbacks + 'callbacks>(
        &mut self,
        label: FuriString,
        number_of_options: u8,
        callbacks: C,
    ) -> VariableItemRef<'callbacks, UniqueCallbackForEachItem<'callbacks>> {
        let mut items_guard = self.context.items.lock();

        unsafe extern "C" fn dispatch_value_changed_callback(raw: *mut sys::VariableItem) {
            let context = unsafe { sys::variable_item_get_context(raw) };
            let context = unsafe { &mut *(context as *mut VariableItemValueCallbacksContext) };
            let item = &context.item;

            let value = unsafe { sys::variable_item_get_current_value_index(raw) };

            let new_label = context.callbacks.get_new_label(item, value);
            unsafe { sys::variable_item_set_current_value_text(raw, new_label.as_c_ptr()) };
            context.value_label = new_label;

            context.callbacks.react_to_change();
        }

        let list_index = items_guard.len();

        let mut item_context = Box::new_uninit();
        let label_ptr = label.as_c_ptr();

        let variable_item = unsafe {
            sys::variable_item_list_add(
                self.as_raw(),
                label_ptr,
                number_of_options,
                Some(dispatch_value_changed_callback),
                Box::as_ptr(&item_context).cast_mut().cast(),
            )
        };

        let inner = unsafe { NonNull::new_unchecked(variable_item) };
        let item = VariableItem {
            inner,
            list_index,
            parent: self.inner.0,
        };

        item_context.write(VariableItemValueCallbacksContext {
            label_ptr,
            callbacks: Box::new(callbacks),
            value_label: FuriString::new(),
            item,
            number_of_options,
        });

        {
            let value_callbacks_context = unsafe { &mut item_context.assume_init_mut() };

            let value_label = value_callbacks_context
                .callbacks
                .get_new_label(&value_callbacks_context.item, 0);
            unsafe {
                sys::variable_item_set_current_value_text(
                    value_callbacks_context.item.inner.as_ptr(),
                    value_label.as_c_ptr(),
                )
            };
            value_callbacks_context.value_label = value_label;
        }

        items_guard.push(VariableItemType::WithValues(unsafe {
            item_context.assume_init()
        }));
        self.context.strings.lock().push(label);

        drop(items_guard);

        VariableItemRef {
            context: self.context.clone(),
            list_index,
        }
    }

    /// Push an item to the end of the variable item list. The item will have a number of options
    /// which can be selected, and, when clicked, will invoke a callback.
    pub fn push_item_with_on_click_callback_and_options<
        C: Callback<'callbacks> + 'callbacks,
        D: OnCurrentValueTextChangedCallbacks + 'callbacks,
    >(
        &mut self,
        label: FuriString,
        on_click_callback: C,
        number_of_options: u8,
        on_current_value_changed_callbacks: D,
    ) -> VariableItemRef<'callbacks, UniqueCallbackForEachItem<'callbacks>> {
        let mut items_guard = self.context.items.lock();

        unsafe extern "C" fn dispatch_value_changed_callback(raw: *mut sys::VariableItem) {
            let context = unsafe { sys::variable_item_get_context(raw) };
            let context = unsafe { &mut *(context as *mut VariableItemValueCallbacksContext) };
            let item = &context.item;

            let value = unsafe { sys::variable_item_get_current_value_index(raw) };

            let new_label = context.callbacks.get_new_label(item, value);
            unsafe { sys::variable_item_set_current_value_text(raw, new_label.as_c_ptr()) };
            context.value_label = new_label;

            context.callbacks.react_to_change();
        }

        let list_index = items_guard.len();

        let mut item_context = Box::new_uninit();
        let label_ptr = label.as_c_ptr();

        let variable_item = unsafe {
            sys::variable_item_list_add(
                self.as_raw(),
                label_ptr,
                number_of_options,
                Some(dispatch_value_changed_callback),
                Box::as_mut_ptr(&mut item_context).cast(),
            )
        };

        let inner = unsafe { NonNull::new_unchecked(variable_item) };
        let item = VariableItem {
            inner,
            list_index,
            parent: self.inner.0,
        };

        item_context.write(VariableItemValueCallbacksContext {
            label_ptr,
            callbacks: Box::new(on_current_value_changed_callbacks),
            value_label: FuriString::new(),
            item,
            number_of_options,
        });

        {
            let value_callbacks_context = unsafe { &mut item_context.assume_init_mut() };

            let value_label = value_callbacks_context
                .callbacks
                .get_new_label(&value_callbacks_context.item, 0);
            unsafe {
                sys::variable_item_set_current_value_text(
                    value_callbacks_context.item.inner.as_ptr(),
                    value_label.as_c_ptr(),
                )
            };
            value_callbacks_context.value_label = value_label;
        }

        items_guard.push(VariableItemType::WithValues(unsafe {
            item_context.assume_init()
        }));
        self.context.strings.lock().push(label);

        drop(items_guard);

        {
            self.context
                .callback
                .lock()
                .0
                .push((list_index, Box::new(on_click_callback)));
        }

        VariableItemRef {
            context: self.context.clone(),
            list_index,
        }
    }

    /// Clear the variable item list.
    ///
    /// All items are cleared, and all callbacks associated with those items will be dropped.
    pub fn clear(&mut self) -> () {
        {
            self.context.items.lock().clear();
            self.context.callback.lock().0.clear();
            self.context.strings.lock().clear();
        }

        unsafe { sys::variable_item_list_reset(self.as_raw()) };
    }
}

impl<'callback, C: Callback<'callback> + 'callback> VariableItemList<'callback, C> {
    /// Creates a new variable item list with a single callback that is invoked whenever any item
    /// is clicked.
    pub fn new_with_callback(on_click_callback: C) -> Self {
        let inner = {
            let variable_item_list = unsafe { sys::variable_item_list_alloc() };
            VariableItemListInner(unsafe { NonNull::new_unchecked(variable_item_list) })
        };

        unsafe extern "C" fn dispatch_callback<'callback, C: Callback<'callback> + 'callback>(
            context: *mut c_void,
            index: u32,
        ) -> () {
            let context = unsafe { &mut *(context as *mut CallbackContext<C>) };

            let item = context.get_item_at_index(index);

            context.callback.lock().on_click(item)
        }

        let callback_context = CallbackContext {
            callback: Mutex::new(on_click_callback),
            strings: Mutex::new(Vec::new()),
            items: Default::default(),
        };

        let res = Self {
            inner,
            context: Arc::new(callback_context),
        };

        unsafe {
            sys::variable_item_list_set_enter_callback(
                res.inner.as_ptr(),
                Some(dispatch_callback::<'callback, C>),
                Arc::as_ptr(&res.context).cast_mut().cast(),
            );
        };

        res
    }

    /// Clear the variable item list.
    ///
    /// Note that this does not have any effect on the callback, which is left unchanged.
    pub fn clear(&mut self) -> () {
        self.context.items.lock().clear();
        self.context.strings.lock().clear();

        unsafe { sys::variable_item_list_reset(self.as_raw()) };
    }
}

impl<'callback, T> VariableItemList<'callback, T> {
    /// Get pointer to the underlying [`sys::VariableItemList`].
    pub fn as_raw(&self) -> *mut sys::VariableItemList {
        self.inner.as_ptr()
    }

    /// Consumes the `VariableItemList`, adding its `View` to the `ViewDispatcher` and returning a
    /// `VariableItemListBoundToViewDispatcher`.
    ///
    /// In the Flipper's codebase, the `VariableItemList` is almost always used alongside a
    /// [`sys::SceneManager`]. However, it is possible to just treat it as any other view, and use
    /// it directly with the `ViewDispatcher`.
    ///
    /// Note that the variable item list does not define a [previous
    /// view](`crate::gui::view::ViewCallbacks::on_back_event`), and so any back events that occur
    /// while this view is current will not be consumed, and will hand control to
    /// [`ViewDispatcherCallbacks::on_navigation`].
    pub fn bind_to_view_dispatcher<'a, 'gui, C: ViewDispatcherCallbacks>(
        self,
        id: u32,
        view_dispatcher: &'a mut ViewDispatcher<'gui, C>,
    ) -> VariableItemListBoundToViewDispatcher<'callback, 'gui, C, T> {
        let raw = unsafe { sys::variable_item_list_get_view(self.inner.as_ptr()) };
        let view = unsafe { View::new_from_raw(raw) };

        match view_dispatcher.add_view(id, view) {
            Ok(view) => {
                VariableItemListBoundToViewDispatcher::<'callback, 'gui, C, T> { inner: self, view }
            }
            Err(_view) => todo!("handle the id already being used"),
        }
    }
}

impl<T> Drop for VariableItemList<'_, T> {
    fn drop(&mut self) {
        self.context.items.lock().clear();
        self.context.strings.lock().clear();

        unsafe { sys::variable_item_list_free(self.as_raw()) };
    }
}

impl VariableItemValueCallbacksContext<'_> {
    /// Note that this will not reset the current option, which may be a value greater than the new
    /// number of options.
    ///
    /// Additionally, this will not trigger a redraw, and so if the "<"/">" characters should be
    /// updated, they will not be.
    pub fn set_number_of_options(&mut self, number_of_options: u8, redraw: bool) -> () {
        let view = unsafe { sys::variable_item_list_get_view(self.item.parent.as_ptr()) };
        // NOTE: this function is necessary to lock the model
        let _ = unsafe { sys::view_get_model(view) };

        crate::trace!(
            "setting number of options for item {:?} to {} (force redraw={})",
            self.label(),
            number_of_options,
            redraw
        );

        self.number_of_options = number_of_options;

        unsafe { sys::view_commit_model(view, redraw) };
    }

    /// WARNING: The value that is set is not bounds checked, and so may be a value greater than
    /// the current number of options.
    ///
    /// Additionally, this will not trigger a redraw, and so the changes to the label will not be
    /// reflected in the UI until the next update, and if the "<"/">" characters should be updated,
    /// they will not be.
    pub fn set_currently_selected_value(
        &mut self,
        value: u8,
        label_override: Option<FuriString>,
        redraw: bool,
    ) -> () {
        let view = unsafe { sys::variable_item_list_get_view(self.item.parent.as_ptr()) };
        // NOTE: this function is necessary to lock the model
        let _ = unsafe { sys::view_get_model(view) };

        let raw = self.item.inner.as_ptr();
        unsafe { sys::variable_item_set_current_value_index(raw, value) };

        if let Some(new_label) = label_override {
            crate::trace!(
                "setting currently selected value for item {:?} to {}, and overriding label to {} (force redraw={})",
                self.label(),
                value,
                new_label,
                redraw
            );

            self.value_label = new_label;

            unsafe { sys::variable_item_set_current_value_text(raw, self.value_label.as_c_ptr()) };
        } else {
            crate::trace!(
                "setting currently selected value for item {:?} to {}, leaving label unchanged (force redraw={})",
                self.label(),
                value,
                redraw
            );
        }

        unsafe { sys::view_commit_model(view, redraw) };
    }

    /// Temporarily override the value label. Note that this will be reset when the user next
    /// changes the value, and [OnCurrentValueTextChangedCallbacks::get_new_label] is called.
    ///
    /// Additionally, this will not trigger a redraw, and so the updated label will not be
    /// reflected in the UI until the next update.
    pub fn override_value_label(&mut self, new_label: FuriString, redraw: bool) -> () {
        let view = unsafe { sys::variable_item_list_get_view(self.item.parent.as_ptr()) };
        // NOTE: this function is necessary to lock the model
        let _ = unsafe { sys::view_get_model(view) };

        crate::trace!(
            "setting label for item {:?} to {} (force redraw={})",
            self.label(),
            new_label,
            redraw
        );

        self.value_label = new_label;

        let raw = self.item.inner.as_ptr();
        unsafe { sys::variable_item_set_current_value_text(raw, self.value_label.as_c_ptr()) };

        unsafe { sys::view_commit_model(view, redraw) };
    }

    fn label<'borrow, 'label>(&'borrow self) -> DebugCStr<'borrow> {
        // SAFETY: this pointer was extracted from a `FuriString`, which lives as long as the
        // `VariableItem`, which is the same lifetime as this `VariableItemValueCallbacksContext`.
        // TODO: reason about aliasing
        let c_str = unsafe { CStr::from_ptr(self.label_ptr) };

        DebugCStr(c_str)
    }
}

struct DebugCStr<'a>(&'a CStr);

impl ufmt::uDebug for DebugCStr<'_> {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        f.write_char('"')?;
        for c in self.0.to_string_lossy().chars() {
            f.write_char(c)?;
        }
        f.write_char('"')
    }
}

/// VariableItemList is usually used alongside a [Scene Manager](`sys::SceneManager`), but may also be used
/// directly.
pub struct VariableItemListBoundToViewDispatcher<
    'callbacks,
    'gui,
    C: ViewDispatcherCallbacks,
    OnClickCallbacks: 'callbacks,
> {
    inner: VariableItemList<'callbacks, OnClickCallbacks>,
    view: ViewDispatcherView<'gui, (), C>,
}

impl<'callbacks, 'gui, VDC: ViewDispatcherCallbacks, OnClickCallbacks: 'callbacks> ufmt::uDebug
    for VariableItemListBoundToViewDispatcher<'callbacks, 'gui, VDC, OnClickCallbacks>
{
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        self.view.fmt(f)
    }
}

impl<'callbacks, 'gui, VDC: ViewDispatcherCallbacks, OnClickCallbacks: 'callbacks>
    AsRef<ViewDispatcherView<'gui, (), VDC>>
    for VariableItemListBoundToViewDispatcher<'callbacks, 'gui, VDC, OnClickCallbacks>
{
    fn as_ref(&self) -> &ViewDispatcherView<'gui, (), VDC> {
        &self.view
    }
}

impl<'callbacks, 'gui, VDC: ViewDispatcherCallbacks, OnClickCallbacks: 'callbacks>
    VariableItemListBoundToViewDispatcher<'callbacks, 'gui, VDC, OnClickCallbacks>
{
    pub fn switch_to_view(&self) -> () {
        self.view.switch_to_view();
    }
}

impl<'callbacks, 'gui, VDC: ViewDispatcherCallbacks, OnClickCallbacks: 'callbacks> Deref
    for VariableItemListBoundToViewDispatcher<'callbacks, 'gui, VDC, OnClickCallbacks>
{
    type Target = VariableItemList<'callbacks, OnClickCallbacks>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'callbacks, 'gui, VDC: ViewDispatcherCallbacks, OnClickCallbacks: 'callbacks> DerefMut
    for VariableItemListBoundToViewDispatcher<'callbacks, 'gui, VDC, OnClickCallbacks>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
