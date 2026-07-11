//! Whisper-style transcription HTTP helpers — ports `pala_note/network.cpp::transcribeOnce`.
//!
//! Provider selection is environment-based: dev uses Cursor API credentials,
//! production uses OpenAI. Cursor does not yet expose `/v1/audio/transcriptions`
//! (returns 404 as of 2026-07); the same multipart format is used so firmware
//! can switch hosts when the route becomes available or via `transcription_base_host`.

use crate::whisper_parse::parse_whisper_text;

pub const TRANSCRIPTION_PATH: &str = "/v1/audio/transcriptions";
pub const DEFAULT_MODEL: &str = "whisper-1";
pub const DEFAULT_BOUNDARY: &str = "----ZoopBoundary";
pub const OPENAI_HOST: &str = "api.openai.com";
pub const CURSOR_HOST: &str = "api.cursor.com";

/// Which upstream credentials and default host to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptionProvider {
    OpenAi,
    Cursor,
}

impl TranscriptionProvider {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" | "openai_whisper" | "whisper" => Some(Self::OpenAi),
            "cursor" => Some(Self::Cursor),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Cursor => "cursor",
        }
    }

    pub fn default_host(self) -> &'static str {
        match self {
            Self::OpenAi => OPENAI_HOST,
            Self::Cursor => CURSOR_HOST,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptionConfig {
    pub provider: TranscriptionProvider,
    pub host: String,
    pub path: String,
    pub api_key: String,
    pub model: String,
    pub boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscribeError {
    UnknownProvider(String),
    MissingApiKey(TranscriptionProvider),
}

impl TranscriptionConfig {
    /// Build config from `secrets.toml` fields (host-testable, no file I/O).
    pub fn from_secrets(
        transcription_provider: &str,
        openai_key: &str,
        cursor_api_key: &str,
        transcription_base_host: Option<&str>,
    ) -> Result<Self, TranscribeError> {
        let provider = TranscriptionProvider::parse(transcription_provider)
            .ok_or_else(|| TranscribeError::UnknownProvider(transcription_provider.to_string()))?;

        let api_key = match provider {
            TranscriptionProvider::OpenAi => openai_key,
            TranscriptionProvider::Cursor => cursor_api_key,
        }
        .trim()
        .to_string();

        if api_key.is_empty() {
            return Err(TranscribeError::MissingApiKey(provider));
        }

        let host = transcription_base_host
            .map(str::trim)
            .filter(|h| !h.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| provider.default_host().to_string());

        Ok(Self {
            provider,
            host,
            path: TRANSCRIPTION_PATH.to_string(),
            api_key,
            model: DEFAULT_MODEL.to_string(),
            boundary: DEFAULT_BOUNDARY.to_string(),
        })
    }

    pub fn transcription_url(&self) -> String {
        format!("https://{}{}", self.host, self.path)
    }

    /// Multipart preamble: model field + file field header (body bytes follow).
    pub fn multipart_preamble(&self, filename: &str) -> String {
        format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"model\"\r\n\r\n\
             {model}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n\
             Content-Type: audio/wav\r\n\r\n",
            boundary = self.boundary,
            model = self.model,
            filename = filename,
        )
    }

    pub fn multipart_epilogue(&self) -> String {
        format!("\r\n--{boundary}--\r\n", boundary = self.boundary)
    }

    pub fn multipart_total_len(&self, file_size: usize, filename: &str) -> usize {
        self.multipart_preamble(filename).len() + file_size + self.multipart_epilogue().len()
    }

    /// HTTP/1.1 request head for a raw TLS socket (matches `network.cpp` layout).
    pub fn http_request_head(&self, total_len: usize) -> String {
        format!(
            "POST {path} HTTP/1.1\r\n\
             Host: {host}\r\n\
             Authorization: Bearer {api_key}\r\n\
             Content-Type: multipart/form-data; boundary={boundary}\r\n\
             Content-Length: {total_len}\r\n\
             Connection: close\r\n\r\n",
            path = self.path,
            host = self.host,
            api_key = self.api_key,
            boundary = self.boundary,
            total_len = total_len,
        )
    }
}

/// Parse Whisper-style JSON `"text"` from an HTTP response body.
pub fn parse_transcription_response(body: &str) -> Option<String> {
    parse_whisper_text(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn openai_config() -> TranscriptionConfig {
        TranscriptionConfig::from_secrets("openai", "sk-test-openai", "", None).unwrap()
    }

    fn cursor_config() -> TranscriptionConfig {
        TranscriptionConfig::from_secrets("cursor", "", "crsr_test_key", None).unwrap()
    }

    #[test]
    fn resolves_openai_provider() {
        let cfg = openai_config();
        assert_eq!(cfg.provider, TranscriptionProvider::OpenAi);
        assert_eq!(cfg.host, OPENAI_HOST);
        assert_eq!(cfg.api_key, "sk-test-openai");
        assert_eq!(cfg.path, "/v1/audio/transcriptions");
        assert_eq!(cfg.model, "whisper-1");
    }

    #[test]
    fn resolves_cursor_provider() {
        let cfg = cursor_config();
        assert_eq!(cfg.provider, TranscriptionProvider::Cursor);
        assert_eq!(cfg.host, CURSOR_HOST);
        assert_eq!(cfg.api_key, "crsr_test_key");
    }

    #[test]
    fn base_host_override() {
        let cfg = TranscriptionConfig::from_secrets(
            "cursor",
            "",
            "crsr_test",
            Some("localhost:8000"),
        )
        .unwrap();
        assert_eq!(cfg.host, "localhost:8000");
    }

    #[test]
    fn rejects_unknown_provider() {
        assert!(matches!(
            TranscriptionConfig::from_secrets("azure", "k", "k", None),
            Err(TranscribeError::UnknownProvider(_))
        ));
    }

    #[test]
    fn rejects_missing_key() {
        assert!(matches!(
            TranscriptionConfig::from_secrets("openai", "", "", None),
            Err(TranscribeError::MissingApiKey(TranscriptionProvider::OpenAi))
        ));
        assert!(matches!(
            TranscriptionConfig::from_secrets("cursor", "", "", None),
            Err(TranscribeError::MissingApiKey(TranscriptionProvider::Cursor))
        ));
    }

    #[test]
    fn multipart_matches_pala_note_shape() {
        let cfg = openai_config();
        let pre = cfg.multipart_preamble("note.wav");
        assert!(pre.contains("name=\"model\""));
        assert!(pre.contains("whisper-1"));
        assert!(pre.contains("filename=\"note.wav\""));
        assert!(pre.contains("Content-Type: audio/wav"));
        assert!(pre.starts_with(&format!("--{}", cfg.boundary)));

        let post = cfg.multipart_epilogue();
        assert_eq!(post, format!("\r\n--{}--\r\n", cfg.boundary));

        assert_eq!(cfg.multipart_total_len(1024, "note.wav"), pre.len() + 1024 + post.len());
    }

    #[test]
    fn http_head_has_bearer_and_host() {
        let cfg = cursor_config();
        let head = cfg.http_request_head(5000);
        assert!(head.contains("POST /v1/audio/transcriptions HTTP/1.1"));
        assert!(head.contains("Host: api.cursor.com"));
        assert!(head.contains("Authorization: Bearer crsr_test_key"));
        assert!(head.contains(&format!("boundary={}", cfg.boundary)));
        assert!(head.contains("Content-Length: 5000"));
    }

    #[test]
    fn parses_response_text() {
        let body = r#"{"text":"Hello from Whisper"}"#;
        assert_eq!(
            parse_transcription_response(body),
            Some("Hello from Whisper".to_string())
        );
    }
}
