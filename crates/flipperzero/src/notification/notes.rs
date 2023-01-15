use super::NotificationMessage;
use flipperzero_sys as sys;

pub static CLICK: NotificationMessage = unsafe { NotificationMessage(sys::message_click) };

pub static C0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_c0) };
pub static CS0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_cs0) };
pub static D0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_d0) };
pub static DS0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_ds0) };
pub static E0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_e0) };
pub static F0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_f0) };
pub static FS0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_fs0) };
pub static G0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_g0) };
pub static GS0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_gs0) };
pub static A0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_a0) };
pub static AS0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_as0) };
pub static B0: NotificationMessage = unsafe { NotificationMessage(sys::message_note_b0) };

pub static C1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_c1) };
pub static CS1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_cs1) };
pub static D1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_d1) };
pub static DS1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_ds1) };
pub static E1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_e1) };
pub static F1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_f1) };
pub static FS1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_fs1) };
pub static G1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_g1) };
pub static GS1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_gs1) };
pub static A1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_a1) };
pub static AS1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_as1) };
pub static B1: NotificationMessage = unsafe { NotificationMessage(sys::message_note_b1) };

pub static C2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_c2) };
pub static CS2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_cs2) };
pub static D2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_d2) };
pub static DS2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_ds2) };
pub static E2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_e2) };
pub static F2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_f2) };
pub static FS2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_fs2) };
pub static G2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_g2) };
pub static GS2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_gs2) };
pub static A2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_a2) };
pub static AS2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_as2) };
pub static B2: NotificationMessage = unsafe { NotificationMessage(sys::message_note_b2) };

pub static C3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_c3) };
pub static CS3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_cs3) };
pub static D3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_d3) };
pub static DS3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_ds3) };
pub static E3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_e3) };
pub static F3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_f3) };
pub static FS3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_fs3) };
pub static G3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_g3) };
pub static GS3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_gs3) };
pub static A3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_a3) };
pub static AS3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_as3) };
pub static B3: NotificationMessage = unsafe { NotificationMessage(sys::message_note_b3) };

pub static C4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_c4) };
pub static CS4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_cs4) };
pub static D4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_d4) };
pub static DS4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_ds4) };
pub static E4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_e4) };
pub static F4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_f4) };
pub static FS4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_fs4) };
pub static G4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_g4) };
pub static GS4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_gs4) };
pub static A4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_a4) };
pub static AS4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_as4) };
pub static B4: NotificationMessage = unsafe { NotificationMessage(sys::message_note_b4) };

pub static C5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_c5) };
pub static CS5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_cs5) };
pub static D5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_d5) };
pub static DS5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_ds5) };
pub static E5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_e5) };
pub static F5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_f5) };
pub static FS5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_fs5) };
pub static G5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_g5) };
pub static GS5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_gs5) };
pub static A5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_a5) };
pub static AS5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_as5) };
pub static B5: NotificationMessage = unsafe { NotificationMessage(sys::message_note_b5) };

pub static C6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_c6) };
pub static CS6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_cs6) };
pub static D6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_d6) };
pub static DS6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_ds6) };
pub static E6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_e6) };
pub static F6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_f6) };
pub static FS6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_fs6) };
pub static G6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_g6) };
pub static GS6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_gs6) };
pub static A6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_a6) };
pub static AS6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_as6) };
pub static B6: NotificationMessage = unsafe { NotificationMessage(sys::message_note_b6) };

pub static C7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_c7) };
pub static CS7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_cs7) };
pub static D7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_d7) };
pub static DS7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_ds7) };
pub static E7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_e7) };
pub static F7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_f7) };
pub static FS7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_fs7) };
pub static G7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_g7) };
pub static GS7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_gs7) };
pub static A7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_a7) };
pub static AS7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_as7) };
pub static B7: NotificationMessage = unsafe { NotificationMessage(sys::message_note_b7) };

pub static C8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_c8) };
pub static CS8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_cs8) };
pub static D8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_d8) };
pub static DS8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_ds8) };
pub static E8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_e8) };
pub static F8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_f8) };
pub static FS8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_fs8) };
pub static G8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_g8) };
pub static GS8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_gs8) };
pub static A8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_a8) };
pub static AS8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_as8) };
pub static B8: NotificationMessage = unsafe { NotificationMessage(sys::message_note_b8) };
