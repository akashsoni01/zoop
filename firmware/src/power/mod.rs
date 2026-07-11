//! Battery ADC and sleep stubs.

use log::info;
use zoop_core::battery::battery_percent_from_adc_samples;
use zoop_core::error::CoreResult;
use zoop_core::io::BatteryAdc;
use zoop_core::sleep::ActivityTimer;

pub struct BatteryMonitor;

impl BatteryMonitor {
    pub fn read_percent(&mut self) -> Option<u8> {
        let samples = self.read_mv_samples(16).ok()?;
        battery_percent_from_adc_samples(&samples)
    }
}

impl BatteryAdc for BatteryMonitor {
    fn read_mv_samples(&mut self, _count: usize) -> CoreResult<Vec<u32>> {
        Ok(vec![2100; 16])
    }
}

pub struct SleepManager {
    pub timer: ActivityTimer,
}

impl SleepManager {
    pub fn new() -> Self {
        Self {
            timer: ActivityTimer::default(),
        }
    }

    pub fn enter_ultra_sleep(&self) {
        info!("sleep: enter_ultra_sleep stub (ext1 wake — HIL pending)");
    }
}

impl Default for SleepManager {
    fn default() -> Self {
        Self::new()
    }
}
