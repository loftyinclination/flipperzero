use crate::bluetooth::profile::{BleProfileCallbacks, Profile};
use crate::error;
use bt_hci::FromHciBytes;
use bt_hci::event::EventPacket;
use core::{ffi::c_void, ptr::NonNull};
use flipperzero_sys as sys;

pub struct EventHandler<'bluetooth, 'profile, PC: BleProfileCallbacks, BEC: BleEventCallbacks> {
    handler: NonNull<sys::GapEventHandler>,
    profile: &'profile Profile<'bluetooth, PC>,
    callbacks: BEC,
}

impl<'bluetooth, 'profile, PC: BleProfileCallbacks, BEC: BleEventCallbacks>
    EventHandler<'bluetooth, 'profile, PC, BEC>
{
    pub fn subscribe(profile: &'profile Profile<'bluetooth, PC>, mut callbacks: BEC) -> Self {
        unsafe extern "C" fn dispatch_ble_event<C: BleEventCallbacks>(
            event: *mut c_void,
            context: *mut c_void,
        ) -> sys::BleEventAckStatus {
            let context = unsafe { &mut *(context as *mut C) };

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
                Ok(event_packet) => match context.handle_event(event_packet) {
                    // NOTE: we don't send out BleEventAckFlowDisable commands because that just
                    // tells the stm32_copro firmware to wait before sending the data again, which
                    // means we'd just end up back here.
                    EventBubbling::Consumed => sys::BleEventAckFlowEnable,
                    EventBubbling::ReturnForAdditionalProcessing => sys::BleEventNotAck,
                },
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

        let handler = unsafe {
            sys::ble_event_dispatcher_register_svc_handler(
                Some(dispatch_ble_event::<BEC>),
                &raw mut callbacks as *mut _,
            )
        };

        let handler = unsafe { NonNull::new_unchecked(handler) };

        Self {
            handler,
            profile,
            callbacks,
        }
    }
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
