use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::bluetooth::hci::HciError;
use crate::bluetooth::profile::{BleProfileCallbacks, Profile};
use crate::furi::sync::Mutex;
use crate::furi::time::FuriDuration;
use crate::{debug, error, warn};
use bt_hci::FromHciBytes;
use bt_hci::event::le::LeEvent;
use bt_hci::event::EventPacket;
use core::{ffi::c_void, ptr::NonNull};
use flipperzero_sys as sys;

struct Context<BEC: BleEventCallbacks> {
    callbacks: BEC,
}

pub struct EventHandler<'bluetooth, 'profile, PC: BleProfileCallbacks, BEC: BleEventCallbacks> {
    handler: NonNull<sys::GapEventHandler>,
    profile: &'profile Profile<'bluetooth, PC>,
    context: Arc<Context<BEC>>,
}

unsafe impl<PC: BleProfileCallbacks, BEC: BleEventCallbacks> Send
    for EventHandler<'_, '_, PC, BEC>
{
}

impl<'bluetooth, 'profile, PC: BleProfileCallbacks, BEC: BleEventCallbacks>
    EventHandler<'bluetooth, 'profile, PC, BEC>
{
    pub fn subscribe(profile: &'profile Profile<'bluetooth, PC>, callbacks: BEC) -> Self {
        unsafe extern "C" fn dispatch_ble_event<C: BleEventCallbacks>(
            event: *mut c_void,
            context: *mut c_void,
        ) -> sys::BleEventAckStatus {
            let context = unsafe { &mut *(context as *mut Context<C>) };
            let callbacks = &mut context.callbacks;

            #[repr(packed, C)]
            struct HciUartPacket {
                kind: u8,
                data: *const (),
            }

            let hci_uart_packet = unsafe { &*event.cast::<HciUartPacket>() };
            let hci_event_packet_ptr = hci_uart_packet.data.cast::<HciEventPacket>();
            let hci_event_packet = unsafe { &*hci_event_packet_ptr };

            #[repr(packed, C)]
            struct HciEventPacket {
                kind: u8,
                len: u8,
                // data: [u8; 1],
            }

            let data_len = hci_event_packet.len as usize;

            let event =
                unsafe { core::slice::from_raw_parts(hci_event_packet_ptr.cast(), 2 + data_len) };

            match EventPacket::from_hci_bytes_complete(event) {
                Ok(event_packet) => {
                    debug!("sending BLE event to handler");

                    let event_handled = callbacks.handle_event(event_packet);

                    debug!("handler finished processing BLE event; {:?}", event_handled);

                    match event_handled {
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
                            error!("failed to parse ble event packet: insufficient data");
                        }
                        bt_hci::FromHciBytesError::InvalidValue => {
                            error!("failed to parse ble event packet: value was out of range");
                        }
                    };
                    sys::BleEventNotAck
                }
            }
        }

        let context = Arc::new(Context {
            callbacks,
        });

        let handler = unsafe {
            sys::ble_event_dispatcher_register_svc_handler(
                Some(dispatch_ble_event::<BEC>),
                Arc::as_ptr(&context).cast_mut().cast(),
            )
        };

        let handler = unsafe { NonNull::new_unchecked(handler) };

        Self {
            handler,
            profile,
            context,
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

#[derive(ufmt::derive::uDebug, Clone, PartialEq, Eq)]
pub enum EventBubbling {
    Consumed,
    ReturnForAdditionalProcessing,
}

pub trait BleEventCallbacks: Send {
    /// Callback to invoke when a BLE event is received.
    ///
    /// Note: this will be invoked on the BLE GAP service thread.
    // TODO: should this take a bt_hci::event::Event enum instead of a packet?
    fn handle_event<'a, 'b>(&'a mut self, event_packet: EventPacket<'b>) -> EventBubbling;
}
