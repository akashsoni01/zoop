//! GPIO button input — ports `buttons.cpp` (HIL pending).

use zoop_core::buttons::ButtonPoller;
use zoop_core::io::Buttons;

use crate::board::config::{BTN_PWR, BTN_REC};

pub struct GpioButtons;

impl GpioButtons {
    pub fn new() -> Self {
        Self
    }
}

impl Buttons for GpioButtons {
    fn rec_pressed(&self) -> bool {
        let _ = BTN_REC;
        false
    }

    fn pwr_pressed(&self) -> bool {
        let _ = BTN_PWR;
        false
    }
}

pub type DeviceButtons = ButtonPoller<GpioButtons>;
