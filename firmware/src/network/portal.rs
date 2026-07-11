//! HTTP transfer portal — `esp-idf-svc` server wrapping `zoop_core::network::portal`.

use log::info;

pub struct TransferPortal {
    pub active: bool,
}

impl TransferPortal {
    pub fn start(&mut self, ip: &str) {
        self.active = true;
        info!("portal: transfer mode at http://{ip}/ (HIL pending)");
    }

    pub fn stop(&mut self) {
        self.active = false;
        info!("portal: stopped");
    }
}

impl Default for TransferPortal {
    fn default() -> Self {
        Self { active: false }
    }
}
