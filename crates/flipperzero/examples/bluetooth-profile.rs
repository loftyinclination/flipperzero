//! Demonstrates how to react to Bluetooth events

#![no_main]
#![no_std]

extern crate alloc;
extern crate flipperzero_alloc;
extern crate flipperzero_rt;

use bt_hci::FromHciBytes;
use bt_hci::event::le::{LeConnectionComplete, LeEventKind, LeEventPacket};
use bt_hci::event::{EventKind, EventPacket};
use core::ffi::CStr;
use flipperzero::bluetooth::Bluetooth;
use flipperzero::bluetooth::handler::{BleEventCallbacks, EventBubbling, EventHandler};
use flipperzero::bluetooth::profile::{
    BleInitialiseProfileCallbacks, BleProfileCallbacks, BleProfileContext, Profile,
};
use flipperzero::gui::view_port::{ViewPort, ViewPortCallbacks};
use flipperzero::print;
use flipperzero_rt::{entry, manifest};

manifest!(name = "Rust Bluetooth Profile example");
entry!(main);

struct Context;

struct ProfileState;

impl BleProfileContext for ProfileState {
    type ProfileContext = Context;
}

impl BleInitialiseProfileCallbacks for ProfileState {
    fn initialise_ble_profile(&mut self) -> Self::ProfileContext {
        Context {}
    }
}

impl BleProfileCallbacks for ProfileState {
    fn configure_name(&self, default_device_name: &'static CStr) -> [u8; 17] {
        let device_name = default_device_name
            .to_str()
            .expect("Device name should be a valid UTF-8 string");

        let device_name = device_name.replace("Flipper", "Test");

        let mut target = [0; 17];
        target.copy_from_slice(&device_name.into_bytes());
        target
    }

    fn configure_appearance(&self) -> u16 {
        bt_hci::uuid::appearance::computer::PALM_SIZE_PCPDA.into()
    }

    fn configure_gap_profile(&mut self, config: &mut flipperzero_sys::GapConfig) {
        config.mac_address[0] ^= 0b10;
        config.mac_address[2] += 2;
    }
}

struct HandlerState;

impl BleEventCallbacks for HandlerState {
    fn handle_event(&mut self, event_packet: EventPacket) -> EventBubbling {
        match event_packet.kind {
            EventKind::Le => {
                let event_packet = LeEventPacket::from_hci_bytes_complete(event_packet.data)
                    .expect("Events originate in the STM32 firmware and should always be valid, and should be received by the flipper in full");
                match event_packet.kind {
                    LeEventKind::LeConnectionComplete => {
                        let connection_complete_event =
                            LeConnectionComplete::from_hci_bytes_complete(event_packet.data)
                                .expect("Events originate in the STM32 firmware and should always be valid, and should be received by the flipper in full");

                        todo!();

                        EventBubbling::ReturnForAdditionalProcessing
                    }
                    LeEventKind::LeAdvertisingReport => {
                        // NOTE: this isn't currently reachable, as advertising reports can only be
                        // received in response to scanning, which is only possible in Central
                        // mode, and the flipper firmware only supports Peripheral connections at
                        // the moment.
                        print!("Advertising report received");
                        EventBubbling::Consumed
                    }
                    _ => EventBubbling::ReturnForAdditionalProcessing,
                }
            }
            EventKind::Vendor => {
                todo!()
            }
            _ => EventBubbling::ReturnForAdditionalProcessing,
        }
    }
}

struct ViewPortState {
    device_broadcasts_received: u32,
}

impl ViewPortCallbacks for ViewPortState {
    fn on_draw(&mut self, canvas: flipperzero::gui::canvas::CanvasView<'_>) {}

    fn on_input(&mut self, event: flipperzero::input::InputEvent) {}
}

fn main(_args: Option<&CStr>) -> i32 {
    let bluetooth = Bluetooth::open();

    let profile = Profile::start(ProfileState {}, &bluetooth);

    let handler = EventHandler::subscribe(&profile, HandlerState {});

    let view_port = ViewPort::new(ViewPortState {
        device_broadcasts_received: 0,
    });

    todo!()
}

#[cfg(miri)]
#[unsafe(no_mangle)]
fn miri_start(_argc: isize, _argv: *const *const u8) -> isize {
    // TODO: Is there any benefit to Miri in hooking up the binary arguments to
    // the test runner?
    main(None).try_into().unwrap_or(isize::MAX)
}
