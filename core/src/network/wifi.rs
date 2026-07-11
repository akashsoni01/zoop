//! WiFi connect policy as pure functions — ports retry UI from `pala_note.ino`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiMode {
    Off,
    Sta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiConnectPhase {
    Idle,
    Connecting { attempt: u32, max_attempts: u32 },
    Connected,
    Failed,
}

/// Sync flow: up to 20 tries × 500 ms (reference).
pub const SYNC_MAX_ATTEMPTS: u32 = 20;
pub const SYNC_RETRY_MS: u64 = 500;

/// Transfer mode: up to 24 tries.
pub const TRANSFER_MAX_ATTEMPTS: u32 = 24;

/// Advance connect state given whether the underlying stack reports connected.
pub fn advance_wifi_connect(
    phase: WifiConnectPhase,
    connected: bool,
    elapsed_ms: u64,
    retry_ms: u64,
    max_attempts: u32,
) -> WifiConnectPhase {
    match phase {
        WifiConnectPhase::Idle => WifiConnectPhase::Connecting {
            attempt: 1,
            max_attempts,
        },
        WifiConnectPhase::Connecting { attempt, max_attempts } => {
            if connected {
                return WifiConnectPhase::Connected;
            }
            if elapsed_ms < retry_ms {
                return WifiConnectPhase::Connecting {
                    attempt,
                    max_attempts,
                };
            }
            if attempt >= max_attempts {
                WifiConnectPhase::Failed
            } else {
                WifiConnectPhase::Connecting {
                    attempt: attempt + 1,
                    max_attempts,
                }
            }
        }
        other => other,
    }
}

/// After sync completes the reference disconnects WiFi.
pub fn post_sync_policy() -> WifiMode {
    WifiMode::Off
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connects_on_success() {
        let p = advance_wifi_connect(
            WifiConnectPhase::Idle,
            true,
            0,
            SYNC_RETRY_MS,
            SYNC_MAX_ATTEMPTS,
        );
        assert!(matches!(
            p,
            WifiConnectPhase::Connecting {
                attempt: 1,
                ..
            }
        ));
        let p2 = advance_wifi_connect(p, true, 500, SYNC_RETRY_MS, SYNC_MAX_ATTEMPTS);
        assert_eq!(p2, WifiConnectPhase::Connected);
    }

    #[test]
    fn fails_after_max_attempts() {
        let mut p = WifiConnectPhase::Connecting {
            attempt: 1,
            max_attempts: 3,
        };
        for i in 1..=3 {
            p = advance_wifi_connect(p, false, 500, 500, 3);
            if i < 3 {
                assert!(matches!(p, WifiConnectPhase::Connecting { .. }));
            }
        }
        assert_eq!(p, WifiConnectPhase::Failed);
    }

    #[test]
    fn transfer_allows_more_attempts() {
        assert_eq!(TRANSFER_MAX_ATTEMPTS, 24);
        assert_eq!(SYNC_MAX_ATTEMPTS, 20);
    }
}
