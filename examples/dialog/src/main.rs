//! Dialog example for Flipper Zero.

#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

// Required for allocator
extern crate flipperzero_alloc;

use core::ffi::CStr;

use flipperzero::furi::canvas::Align;
use flipperzero::furi::dialog::{self, DialogMessage, DialogMessageButton, DialogsApp};
use flipperzero_rt::{entry, manifest};

manifest!(name = "Rust dialog example");
entry!(main);

fn main(_args: *mut u8) -> i32 {
    // To customize the dialog, use the DialogMessage API:
    let mut dialogs = DialogsApp::open();
    let mut message = DialogMessage::new();

    message.set_header(
        CStr::from_bytes_with_nul(b"Make your move!\0").unwrap(),
        0,
        0,
        Align::Left,
        Align::Top,
    );
    message.set_text(
        CStr::from_bytes_with_nul(b"Choose one of the following:\0").unwrap(),
        0,
        26,
        Align::Left,
        Align::Top,
    );
    message.set_buttons(
        Some(CStr::from_bytes_with_nul(b"Rock\0").unwrap()),
        Some(CStr::from_bytes_with_nul(b"Paper\0").unwrap()),
        Some(CStr::from_bytes_with_nul(b"Scissor\0").unwrap()),
    );

    let button = dialogs.show(&message);

    // ... or use dialog::alert() to display a simple message:
    match button {
        DialogMessageButton::Left => dialog::alert("You chose Rock..."),
        DialogMessageButton::Center => dialog::alert("You chose Paper..."),
        DialogMessageButton::Right => dialog::alert("You chose Scissors..."),
        DialogMessageButton::Back => dialog::alert("You chose not to play..."),
    }

    dialog::alert("... but dolphins can't play rock paper scissors anyways :)");

    0
}
