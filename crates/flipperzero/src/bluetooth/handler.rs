use alloc::boxed::Box;

use crate::bluetooth::profile::{BleProfileCallbacks, Profile};
use crate::furi::event_flag::EventFlag;
use crate::furi::time::FuriDuration;
use crate::{error, warn};
use bt_hci::FromHciBytes;
use bt_hci::event::le::LeEvent;
use bt_hci::event::{EventPacket, EventParams};
use core::marker::PhantomData;
use core::{ffi::c_void, ptr::NonNull};
use flipperzero_sys as sys;

const FLAGS: u32 = 1;

struct Context<BEC: BleEventCallbacks> {
    callbacks: BEC,
    event_flag: EventFlag,
    pattern_override: Option<Box<dyn ResponsePattern>>,
}

pub struct EventHandler<'bluetooth, 'profile, PC: BleProfileCallbacks, BEC: BleEventCallbacks> {
    handler: NonNull<sys::GapEventHandler>,
    profile: &'profile Profile<'bluetooth, PC>,
    // TODO: maybe lock?
    context: Context<BEC>,
}

impl<'bluetooth, 'profile, PC: BleProfileCallbacks, BEC: BleEventCallbacks>
    EventHandler<'bluetooth, 'profile, PC, BEC>
{
    pub fn subscribe(profile: &'profile Profile<'bluetooth, PC>, mut callbacks: BEC) -> Self {
        unsafe extern "C" fn dispatch_ble_event<C: BleEventCallbacks>(
            event: *mut c_void,
            context: *mut c_void,
        ) -> sys::BleEventAckStatus {
            let context = unsafe { &mut *(context as *mut Context<C>) };
            let callbacks = &mut context.callbacks;

            // event here is a hci_uart_pckt, and so the first byte is the type of that uart packet
            let data_len = {
                let data_len_ptr = unsafe { event.offset(2) };
                let data_len = unsafe { *data_len_ptr.cast::<u8>() };
                data_len as usize
            };

            let event: &[u8] = {
                let complete_data_ptr = unsafe { event.offset(1) };
                unsafe {
                    core::slice::from_raw_parts(
                        // offset 1 to get past the uart_pckt Type field
                        complete_data_ptr.cast::<u8>(),
                        // add 2, as this is plen, and we want to include evt and plen in the EventPacket that
                        // is parsed
                        data_len + 2,
                    )
                }
            };

            match EventPacket::from_hci_bytes_complete(event) {
                Ok(event_packet) => {
                    if let Some(pattern) = &context.pattern_override {
                        todo!()
                    }

                    match callbacks.handle_event(event_packet) {
                        // NOTE: we don't send out BleEventAckFlowDisable commands because that just
                        // tells the stm32_copro firmware to wait before sending the data again, which
                        // means we'd just end up back here.
                        EventBubbling::Consumed => sys::BleEventAckFlowEnable,
                        EventBubbling::ReturnForAdditionalProcessing => sys::BleEventNotAck,
                    }
                }
                Err(error) => {
                    match error {
                        bt_hci::FromHciBytesError::InvalidSize => {
                            error!("failed to parse event packet: insufficient data");
                        }
                        bt_hci::FromHciBytesError::InvalidValue => {
                            error!("failed to parse event packet: value was out of range");
                        }
                    };
                    sys::BleEventNotAck
                }
            }
        }

        let mut context = Context {
            callbacks,
            event_flag: Default::default(),
            pattern_override: None,
        };

        let handler = unsafe {
            sys::ble_event_dispatcher_register_svc_handler(
                Some(dispatch_ble_event::<BEC>),
                (&raw mut context).cast()
            )
        };

        let handler = unsafe { NonNull::new_unchecked(handler) };

        Self {
            handler,
            profile,
            context,
        }
    }

    pub(crate) fn wait_and_consume_response(&mut self, response: impl ResponsePattern + 'static) {
        assert!(
            self.context
                .pattern_override
                .replace(Box::new(response))
                .is_none()
        );

        match self
            .context
            .event_flag
            .wait_any_flags(FLAGS, true, FuriDuration::WAIT_FOREVER)
        {
            Ok(_) => (),
            Err(status) => warn!("Errored out when waiting for BLE event: {}", status),
        }
    }
}

pub(crate) struct PatternMatcher<T> {
    phantom: PhantomData<T>,
}

impl<T> PatternMatcher<T> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<'a, T: EventParams<'a>> ResponsePattern for PatternMatcher<T> {
    fn is_of_type(&self, event: &EventPacket<'_>) -> bool {
        todo!()
    }
}

trait ResponsePattern {
    fn is_of_type(&self, event: &EventPacket<'_>) -> bool;
}

impl<'bluetooth, 'profile, PC: BleProfileCallbacks, BEC: BleEventCallbacks> Drop
    for EventHandler<'bluetooth, 'profile, PC, BEC>
{
    fn drop(&mut self) {
        unsafe { sys::ble_event_dispatcher_unregister_svc_handler(self.handler.as_ptr()) }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum EventBubbling {
    Consumed,
    ReturnForAdditionalProcessing,
}

pub trait BleEventCallbacks: Send {
    /// Callback to invoke when a BLE event is received.
    ///
    /// Note: this will be invoked on the BLE GAP service thread.
    fn handle_event<'a, 'b>(&'a mut self, event_packet: EventPacket<'b>) -> EventBubbling;
}
