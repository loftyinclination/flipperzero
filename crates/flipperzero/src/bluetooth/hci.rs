use bt_hci::{
    FromHciBytes, FromHciBytesError, WriteHci,
    cmd::{AsyncCmd, Cmd, CmdReturnBuf, SyncCmd},
    param::{RemainingBytes, Status},
};
use flipperzero_sys as sys;

pub use bt_hci::cmd;

pub enum HciError {
    Status(Status),
    InvalidSize,
    InvalidValue,
}

impl From<FromHciBytesError> for HciError {
    fn from(value: FromHciBytesError) -> Self {
        match value {
            FromHciBytesError::InvalidSize => HciError::InvalidSize,
            FromHciBytesError::InvalidValue => HciError::InvalidValue,
        }
    }
}

/// Sends a HCI command to the STM32WB55's M0 Coprocessor, and transforms the response.
///
/// NOTE: The [Command Complete](bt_hci::event::CommandComplete) event is processed by the STM32
/// Copro library.
pub fn send_hci_command<C: SyncCmd>(payload: C) -> Result<C::Return, HciError> {
    let opcode = <C as Cmd>::OPCODE;
    let params = payload.params();
    let mut retval = C::ReturnBuf::new();

    let cparam = {
        let data = alloc::boxed::Box::<[u8]>::new_zeroed_slice(params.size());
        let mut data = unsafe { data.assume_init() };

        {
            let data: &mut [u8] = &mut data;
            params
                .write_hci(data)
                .expect("The slice was created with the size specified for the event");
        }

        data
    };

    let mut hci_request = sys::hci_request {
        ogf: opcode.group().to_raw() as u16,
        ocf: opcode.cmd(),
        event: 0, // UNUSED
        cparam: cparam.as_ptr().cast_mut().cast(),
        clen: params.size() as i32,
        rparam: (&raw mut retval).cast(),
        rlen: C::ReturnBuf::LEN as i32,
    };

    crate::trace!(
        "Sending HCI command {:?} and waiting for CommandComplete response",
        hci_request
    );

    let status = (unsafe { flipperzero_sys::hci_send_req(&raw mut hci_request, 0) } as u8).into();

    crate::trace!(
        "Got response from HCI command {:?}: status={:?}",
        hci_request,
        status
    );

    if status != Status::SUCCESS {
        return Err(HciError::Status(status));
    }

    let return_param_bytes =
        RemainingBytes::from_hci_bytes_complete(&retval.as_ref()[..C::ReturnBuf::LEN])
            .map_err(HciError::from)?;

    C::Return::from_hci_bytes_complete(&return_param_bytes).map_err(HciError::from)
}

/// Sends a HCI command to the STM32WB55's M0 Coprocessor.
///
/// NOTE: The [Command Status](bt_hci::event::CommandStatus) event is processed by the STM32
/// Copro library.
pub fn send_async_hci_command<C: AsyncCmd>(payload: C) -> Result<(), Status> {
    let opcode = <C as Cmd>::OPCODE;
    let params = payload.params();

    let cparam = {
        let data = alloc::boxed::Box::<[u8]>::new_zeroed_slice(params.size());
        let mut data = unsafe { data.assume_init() };

        {
            let data: &mut [u8] = &mut data;
            params
                .write_hci(data)
                .expect("The slice was created with the size specified for the event");
        }

        data
    };

    let mut status: Status = Status::new(0);

    let mut hci_request = sys::hci_request {
        ogf: opcode.group().to_raw() as u16,
        ocf: opcode.cmd(),
        event: 0, // UNUSED
        cparam: cparam.as_ptr().cast_mut().cast(),
        clen: params.size() as i32,
        rparam: (&raw mut status).cast(),
        rlen: 1,
    };

    crate::trace!(
        "Sending HCI command {:?} and waiting for CommandStatus response",
        hci_request
    );

    let send_status =
        (unsafe { flipperzero_sys::hci_send_req(&raw mut hci_request, 0) } as u8).into();

    crate::trace!(
        "Got response from HCI command {:?}: send_status={}, status={}",
        hci_request,
        send_status,
        status
    );

    if send_status != Status::SUCCESS {
        return Err(send_status);
    }

    if status != Status::SUCCESS {
        return Err(status);
    }

    Ok(())
}

#[cfg(miri)]
pub mod miri {
    #[macro_export]
    macro_rules! receive {
        ($item:expr, event from $bt:ident) => {{
            use bt_hci::FixedSizeValue;
            use bt_hci::event::EventParams;
            use core::ptr;
            use flipperzero_sys::bt_inner::HciEventPacket;

            let item = $item;

            let mut bt = $bt.lock("send bt message");

            {
                let msg = alloc::format!("Sending Bluetooth event: {}\n", stringify!($key));

                unsafe extern "Rust" {
                    pub safe fn miri_write_to_stdout(bytes: &[u8]);
                }
                miri_write_to_stdout(msg.as_bytes());
            }

            fn get_event_code<'a, T: EventParams<'a>>(_t: &T) -> u8 {
                T::EVENT_CODE
            }

            let item = HciEventStruct {
                kind: get_event_code(&item),
                len: core::mem::size_of_val(&item) as u8,
                data: ptr::from_ref(item).cast(),
            };

            flipperzero_sys::BtInner::receive_hci_event(&mut bt, item);
        }};
        ($event:expr, le event from $bt:ident) => {{
            extern crate alloc;

            use bt_hci::event::le::LeEventParams;
            use bt_hci::event::{EventKind, EventParams};
            use bt_hci::{FixedSizeValue, WriteHci};
            use core::{ffi::c_void, ops::DerefMut, ptr};
            use flipperzero_sys::bt_inner::HciEventPacket;

            unsafe extern "Rust" {
                pub safe fn miri_write_to_stdout(bytes: &[u8]);
            }

            let le_event = $event;

            let mut bt = $bt.lock("send bt message");

            fn get_subevent_code<'a, T: LeEventParams<'a>>(_t: &T) -> u8 {
                T::SUBEVENT_CODE
            }

            let subevent_kind = get_subevent_code(&le_event);

            miri_write_to_stdout(
                alloc::format!(
                    "Sending Bluetooth LE event: type={}, size={}, {:?}\n",
                    subevent_kind,
                    core::mem::size_of_val(&le_event) + 1,
                    le_event
                )
                .as_bytes(),
            );

            #[repr(packed, C)]
            struct ConstructableHciLeEventPacket {
                subevent_kind: u8,
                data: *const c_void,
            }

            let data = alloc::boxed::Box::<[u8]>::new_zeroed_slice(le_event.size());
            let mut data = unsafe { data.assume_init() };

            le_event
                .write_hci(data.deref_mut())
                .expect("The slice was created with the size specified for the event");

            let le_item = ConstructableHciLeEventPacket {
                subevent_kind,
                data: (&raw const data).cast(),
            };

            let event = HciEventPacket {
                kind: EventKind::Le.0,
                len: 2,
                data: (&raw const le_item).cast(),
            };

            flipperzero_sys::BtInner::receive_hci_event(&mut bt, &raw const event);

            miri_write_to_stdout(b"Finished sending BLuetooth LE event\n");
        }};
    }

    pub use receive;
}
