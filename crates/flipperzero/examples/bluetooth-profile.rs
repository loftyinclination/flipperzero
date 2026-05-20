//! Demonstrates how to react to Bluetooth events

#![no_main]
#![no_std]
#![feature(box_as_ptr)]

extern crate alloc;
extern crate flipperzero_alloc;
extern crate flipperzero_rt;

use alloc::sync::Arc;
use bt_hci::FromHciBytes;
use bt_hci::event::le::{LeAdvertisingReport, LeConnectionComplete, LeEvent, LeEventPacket};
use bt_hci::event::{EventKind, EventPacket};
use core::ffi::CStr;
use flipperzero::bluetooth::Bluetooth;
use flipperzero::bluetooth::handler::{BleEventCallbacks, EventBubbling, EventHandler};
use flipperzero::bluetooth::profile::{
    BleInitialiseProfileCallbacks, BleProfileCallbacks, BleProfileContext,
    FURI_HAL_VERSION_DEVICE_NAME_LENGTH, GapConfig, Profile,
};
use flipperzero::gui::Gui;
use flipperzero::gui::view_port::{ViewPort, ViewPortCallbacks};
use flipperzero::println;
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
    fn configure_name(
        &self,
        default_device_name: &'static CStr,
    ) -> [u8; FURI_HAL_VERSION_DEVICE_NAME_LENGTH - 1] {
        let device_name = default_device_name
            .to_str()
            .expect("Device name should be a valid UTF-8 string");

        let device_name = device_name.replace("Flipper", "Test");

        let mut target = [0; FURI_HAL_VERSION_DEVICE_NAME_LENGTH - 1];
        let bytes = device_name.into_bytes();
        target[..core::cmp::min(FURI_HAL_VERSION_DEVICE_NAME_LENGTH - 1, bytes.len())]
            .copy_from_slice(bytes.as_slice());
        target
    }

    fn configure_appearance(&self) -> u16 {
        bt_hci::uuid::appearance::computer::PALM_SIZE_PCPDA.into()
    }

    fn configure_gap_profile(&mut self, config: &mut GapConfig) {
        config.mac_address[0] ^= 0b10;
        config.mac_address[2] += 2;
    }
}

struct HandlerState;

impl BleEventCallbacks for HandlerState {
    fn handle_event(&mut self, event_packet: EventPacket) -> EventBubbling {
        match event_packet.kind {
            EventKind::Le => {
                let le_event = LeEventPacket::from_hci_bytes_complete(event_packet.data)
                    .expect("Events originate in the STM32 firmware and should always be valid, and should be received by the flipper in full")
                    .try_into()
                    .expect("All events should be parsable into LeEvent");

                match le_event {
                    LeEvent::LeConnectionComplete(data) => {
                        self.handle_connection_complete_event(data)
                    }
                    LeEvent::LeAdvertisingReport(reports) => {
                        self.handle_advertising_report(reports)
                    }
                    _ => EventBubbling::ReturnForAdditionalProcessing,
                }
            }
            EventKind::Vendor => {
                todo!("vendor specific event")
            }
            _ => EventBubbling::ReturnForAdditionalProcessing,
        }
    }
}

impl HandlerState {
    fn handle_advertising_report(&mut self, reports: LeAdvertisingReport<'_>) -> EventBubbling {
        // NOTE: this isn't currently reachable, as advertising reports can only be
        // received in response to scanning, which is only possible in Central
        // mode, and the flipper firmware only supports Peripheral connections at
        // the moment.

        for report in reports.reports.iter() {
            println!("Advertising report received");
        }
        EventBubbling::Consumed
    }

    fn handle_connection_complete_event(&mut self, data: LeConnectionComplete) -> EventBubbling {
        todo!("connection complete");

        EventBubbling::ReturnForAdditionalProcessing
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

    let _handler = EventHandler::subscribe(&profile, HandlerState {});

    let view_port = ViewPort::new(ViewPortState {
        device_broadcasts_received: 0,
    });

    let gui = Gui::open();

    #[cfg(miri)]
    let (miri_gui, miri_bt) = {
        unsafe { Arc::increment_strong_count(gui.as_ptr()) };
        let miri_gui: Arc<flipperzero_sys::Gui> = unsafe { Arc::from_raw(gui.as_ptr()) };

        unsafe { Arc::increment_strong_count(bluetooth.as_ptr()) };
        let miri_bt = unsafe { Arc::from_raw(bluetooth.as_ptr()) };

        (miri_gui, miri_bt)
    };

    let _view_port = gui.add_view_port(view_port, flipperzero::gui::GuiLayer::Fullscreen);

    #[cfg(miri)]
    let status = run_until_exit_miri(miri_gui, miri_bt);

    status
}

#[cfg(miri)]
fn run_until_exit_miri(gui: Arc<flipperzero_sys::Gui>, bt: Arc<flipperzero_sys::Bt>) -> i32 {
    use flipperzero::bluetooth::bt_hci::event::le::LeAdvertisingReport;
    use flipperzero::bluetooth::hci::miri::receive;
    use flipperzero::input::miri::send;

    {
        let reports = bt_hci::param::LeAdvReports::default();

        receive!(LeAdvertisingReport{ reports }, le event from bt);
    }

    {
        let reports = bt_hci::param::LeAdvReports::default();

        let _ = receive!(LeAdvertisingReport{ reports }, le event from bt);
    }

    send!(Back event to gui); // leave

    0
}

#[cfg(miri)]
#[unsafe(no_mangle)]
fn miri_start(_argc: isize, _argv: *const *const u8) -> isize {
    // TODO: Is there any benefit to Miri in hooking up the binary arguments to
    // the test runner?
    main(None).try_into().unwrap_or(isize::MAX)
}
