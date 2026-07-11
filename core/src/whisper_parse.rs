//! Minimal Whisper JSON `"text"` extraction — ports `network.cpp::parseWhisperText`.

/// Extract the `"text":"..."` field from a Whisper API JSON response.
pub fn parse_whisper_text(resp: &str) -> Option<String> {
    let marker = "\"text\":\"";
    let start = resp.find(marker)? + marker.len();
    let mut end = start;
    let bytes = resp.as_bytes();
    while end < bytes.len() {
        if bytes[end] == b'\\' && end + 1 < bytes.len() {
            end += 2;
            continue;
        }
        if bytes[end] == b'"' {
            break;
        }
        end += 1;
    }
    if end >= bytes.len() {
        return None;
    }
    let mut text = String::new();
    let mut i = start;
    while i < end {
        if bytes[i] == b'\\' && i + 1 < end {
            let nx = bytes[i + 1];
            match nx {
                b'"' => text.push('"'),
                b'\\' => text.push('\\'),
                b'n' => text.push(' '),
                _ => text.push(nx as char),
            }
            i += 2;
        } else {
            text.push(bytes[i] as char);
            i += 1;
        }
    }
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_json() {
        let json = r#"{"text":"Hello world"}"#;
        assert_eq!(parse_whisper_text(json), Some("Hello world".to_string()));
    }

    #[test]
    fn unescapes_quotes_and_newlines() {
        let json = r#"{"text":"Say \"hi\"\nand bye"}"#;
        assert_eq!(
            parse_whisper_text(json),
            Some("Say \"hi\" and bye".to_string())
        );
    }

    #[test]
    fn returns_none_when_missing() {
        assert_eq!(parse_whisper_text(r#"{"error":"x"}"#), None);
    }

    #[test]
    fn handles_whitespace_in_text() {
        let json = r#"{"text":"  meeting at three  "}"#;
        assert_eq!(
            parse_whisper_text(json),
            Some("  meeting at three  ".to_string())
        );
    }
}
