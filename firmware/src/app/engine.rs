//! Firmware app engine — wires BSP stubs to `zoop_core::App` (loop body pending SD/HIL).

use log::info;
use zoop_core::app::App;
use zoop_core::buttons::ButtonPoller;
use zoop_core::error::CoreResult;
use zoop_core::io::Clock;

use crate::audio::Es8311Audio;
use crate::board::rtc::RtcChip;
use crate::display::EpaperDisplay;
use crate::input::buttons::{DeviceButtons, GpioButtons};
use crate::network::ntp::NtpClient;
use crate::network::portal::TransferPortal;
use crate::network::whisper::WhisperClient;
use crate::network::wifi::WifiManager;
use crate::power::sleep::SleepManager;
use crate::power::BatteryMonitor;
use crate::storage::SdStorage;

/// Monotonic ms since boot (stub until esp-idf timer wired).
pub struct EspClock {
    pub now_ms: u64,
}

impl Clock for EspClock {
    fn now_ms(&self) -> u64 {
        self.now_ms
    }
}

/// Firmware RTC + NTP time source stub.
pub struct FirmwareTime {
    pub utc: Option<String>,
    pub ntp: NtpClient,
    pub rtc: RtcChip,
}

impl zoop_core::io::TimeSource for FirmwareTime {
    fn utc_iso(&self) -> Option<String> {
        self.utc.clone()
    }

    fn set_utc_iso(&mut self, iso: &str) -> CoreResult<()> {
        self.utc = Some(iso.to_string());
        self.rtc.write_utc_iso(iso);
        Ok(())
    }
}

/// Holds all BSP modules and core app state.
pub struct FirmwareEngine {
    pub storage: SdStorage,
    pub display: EpaperDisplay,
    pub audio: Es8311Audio,
    pub buttons: DeviceButtons,
    pub battery: BatteryMonitor,
    pub sleep: SleepManager,
    pub wifi: WifiManager,
    pub portal: TransferPortal,
    pub whisper: WhisperClient,
    pub time: FirmwareTime,
    pub clock: EspClock,
}

impl FirmwareEngine {
    pub fn new(
        storage: SdStorage,
        display: EpaperDisplay,
        audio: Es8311Audio,
        rtc: RtcChip,
        whisper: WhisperClient,
    ) -> Self {
        Self {
            storage,
            display,
            audio,
            buttons: ButtonPoller::new(GpioButtons::new()),
            battery: BatteryMonitor,
            sleep: SleepManager::new(),
            wifi: WifiManager::new(),
            portal: TransferPortal::default(),
            whisper,
            time: FirmwareTime {
                utc: None,
                ntp: NtpClient::new(),
                rtc,
            },
            clock: EspClock { now_ms: 0 },
        }
    }

    pub fn boot(&mut self) -> CoreResult<()> {
        let Self {
            storage,
            display,
            audio,
            time,
            battery,
            clock,
            ..
        } = self;
        let mut app = App::new(storage, display, audio, time, battery);
        app.boot(clock)?;
        info!(
            "app: boot state={:?} payments={}",
            app.state.state(),
            app.ledger.len()
        );
        Ok(())
    }

    pub fn tick(&mut self) -> CoreResult<()> {
        self.clock.now_ms = self.clock.now_ms.saturating_add(20);
        let now_ms = self.clock.now_ms;
        let Self {
            storage,
            display,
            audio,
            time,
            battery,
            buttons,
            sleep,
            clock,
            ..
        } = self;
        let mut app = App::new(storage, display, audio, time, battery);
        app.tick(buttons, clock)?;
        if sleep.should_sleep(app.state.state(), now_ms) {
            sleep.enter_ultra_sleep();
        }
        Ok(())
    }
}
