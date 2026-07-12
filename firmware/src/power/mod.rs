//! Battery ADC and sleep stubs.

use log::info;
use zoop_core::battery::battery_percent_from_adc_samples;
use zoop_core::error::CoreResult;
use zoop_core::io::BatteryAdc;

use crate::board::config::BAT_ADC_PIN;

pub mod sleep;

pub use sleep::SleepManager;

pub struct BatteryMonitor;

impl BatteryMonitor {
    pub fn read_percent(&mut self) -> Option<u8> {
        let samples = self.read_mv_samples(16).ok()?;
        battery_percent_from_adc_samples(&samples)
    }
}

impl Default for BatteryMonitor {
    fn default() -> Self {
        Self
    }
}

impl BatteryAdc for BatteryMonitor {
    fn read_mv_samples(&mut self, _count: usize) -> CoreResult<Vec<u32>> {
        info!("battery: ADC stub GPIO{BAT_ADC_PIN}");
        Ok(vec![2100; 16])
    }
}
