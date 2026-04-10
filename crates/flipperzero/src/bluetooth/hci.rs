use bt_hci::{
    FromHciBytes, FromHciBytesError, WriteHci,
    cmd::{Cmd, CmdReturnBuf, SyncCmd},
    param::{RemainingBytes, Status},
};
use core::ptr;
use flipperzero_sys as sys;

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

pub fn send_hci_command<C: SyncCmd>(payload: C) -> Result<C::Return, HciError> {
    let opcode = <C as Cmd>::OPCODE;
    let params = payload.params();
    let mut retval = C::ReturnBuf::new();

    let mut hci_request = sys::hci_request {
        ogf: opcode.group().to_raw() as u16,
        ocf: opcode.cmd(),
        event: 0, // UNUSED
        cparam: ptr::from_ref(params).cast_mut().cast(),
        clen: params.size() as i32,
        rparam: (&raw mut retval).cast(),
        rlen: C::ReturnBuf::LEN as i32,
    };

    let status = (unsafe { flipperzero_sys::hci_send_req(&raw mut hci_request, 0) } as u8).into();
    if status != Status::SUCCESS {
        return Err(HciError::Status(status));
    }

    let return_param_bytes =
        RemainingBytes::from_hci_bytes_complete(&retval.as_ref()[..C::ReturnBuf::LEN])
            .map_err(HciError::from)?;

    C::Return::from_hci_bytes_complete(&return_param_bytes).map_err(HciError::from)
}
