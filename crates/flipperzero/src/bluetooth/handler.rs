use crate::bluetooth::Bluetooth;
use core::{ffi::c_void, marker::PhantomData};
use flipperzero_sys as sys;
use sys::BleEventAckStatus;

pub struct EventHandler<'a, C: BleEventCallbacks> {
    callbacks: C,
    _phantom: PhantomData<&'a Bluetooth>,
}

impl<'a, C: BleEventCallbacks> EventHandler<'a, C> {
    fn subscribe(_bluetooth: &'a Bluetooth, mut callbacks: C) -> Self {
        unsafe extern "C" fn dispatch_ble_event<C: BleEventCallbacks>(
            event: *mut c_void,
            context: *mut c_void,
        ) -> BleEventAckStatus {
            // event here is a hci_uart_pckt, and so the first byte is the type of that uart packet
            let data_len = {
                let data_len_ptr = unsafe { event.offset(2) };
                let data_len = unsafe { *data_len_ptr.cast::<u8>() };
                data_len as usize
            };

            let event: &[u8] = {
                let complete_data_ptr = unsafe { event_ptr.offset(1) };
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

            let event_packet = {
                match EventPacket::from_hci_bytes_complete(event) {
                    Ok(event_packet) => event_packet,
                    Err(error) => {
                        match error {
                            bt_hci::FromHciBytesError::InvalidSize => {
                                error!("failed to parse event packet: insufficient data");
                            }
                            bt_hci::FromHciBytesError::InvalidValue => {
                                error!("failed to parse event packet: value was out of range");
                            }
                        };
                        return sys::BleEventNotAck;
                    }
                }
            };

            todo!()
        }

        unsafe {
            sys::ble_event_dispatcher_register_svc_handler(
                Some(dispatch_ble_event::<C>),
                &raw mut callbacks as *mut _,
            )
        };

        Self {
            callbacks,
            _phantom: PhantomData,
        }
    }
}

trait BleEventCallbacks {}
