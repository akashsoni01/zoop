//! UI sound policy — ports `sounds.h` toggle.

use crate::io::{Audio, SoundKind};

/// Logs or plays UI beeps; respects enabled flag from settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundsPolicy {
    pub enabled: bool,
    pub events: Vec<SoundKind>,
}

impl SoundsPolicy {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            events: Vec::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn play(&mut self, audio: &mut impl Audio, kind: SoundKind) {
        self.events.push(kind);
        if self.enabled {
            audio.play_beep(kind);
        }
    }

    pub fn last_event(&self) -> Option<SoundKind> {
        self.events.last().copied()
    }

    pub fn clear_log(&mut self) {
        self.events.clear();
    }
}

impl Default for SoundsPolicy {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockAudio;

    #[test]
    fn disabled_sounds_log_only() {
        let mut policy = SoundsPolicy::new(false);
        let mut audio = MockAudio::new();
        policy.play(&mut audio, SoundKind::Select);
        assert_eq!(policy.last_event(), Some(SoundKind::Select));
        assert!(audio.beep_log.is_empty());
    }

    #[test]
    fn enabled_sounds_forward_to_audio() {
        let mut policy = SoundsPolicy::new(true);
        let mut audio = MockAudio::new();
        policy.play(&mut audio, SoundKind::Saved);
        assert_eq!(audio.beep_log, vec![SoundKind::Saved]);
    }
}
