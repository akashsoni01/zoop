//! Portal HTML helpers — ports `network.cpp` `htmlEscape` and export formatting.

pub const EXPORT_MAX_BYTES: usize = 55_000;

/// Escape HTML special characters for portal output.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportNote {
    pub num: i32,
    pub tag: String,
    pub created_utc: Option<String>,
    pub transcript: String,
    pub has_text: bool,
}

/// Build bulk `.txt` export — mirrors `handleExportTxt()` with 55 KB truncation.
pub fn format_export_text(filter: &str, notes: &[ExportNote]) -> String {
    let mut export = format!("Zoop Export\nFilter: {filter}\n------------------------------\n\n");
    for note in notes.iter().rev() {
        if filter != "All" && filter != note.tag {
            continue;
        }
        let transcript = if note.transcript.is_empty() {
            if note.has_text {
                "(empty transcript)".to_string()
            } else {
                "Not transcribed yet.".to_string()
            }
        } else {
            note.transcript.clone()
        };
        export.push_str(&format_note_heading(note.num, &note.tag));
        if let Some(ref utc) = note.created_utc {
            if !utc.is_empty() {
                export.push_str(utc);
                export.push('\n');
            }
        }
        export.push('\n');
        export.push_str(&transcript);
        export.push_str("\n\n------------------------------\n\n");
        if export.len() > EXPORT_MAX_BYTES {
            export.push_str("\nExport truncated on device because it became too large.\n");
            break;
        }
    }
    export
}

pub fn format_note_heading(num: i32, tag: &str) -> String {
    format!("#{num:03} · {tag}\n")
}

pub fn export_filename(filter: &str) -> String {
    if filter == "All" {
        "zoop_notes_export.txt".to_string()
    } else {
        format!("zoop_notes_export_{filter}.txt")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_escape_encodes_special_chars() {
        assert_eq!(
            html_escape(r#"<a href="x">&"y"</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&quot;y&quot;&lt;/a&gt;"
        );
    }

    #[test]
    fn export_includes_notes_newest_first() {
        let notes = vec![
            ExportNote {
                num: 1,
                tag: "Work".into(),
                created_utc: Some("2026-01-01T10:00:00Z".into()),
                transcript: "First".into(),
                has_text: true,
            },
            ExportNote {
                num: 2,
                tag: "Idea".into(),
                created_utc: None,
                transcript: "Second".into(),
                has_text: true,
            },
        ];
        let text = format_export_text("All", &notes);
        assert!(text.contains("#002 · Idea"));
        assert!(text.contains("#001 · Work"));
        let pos2 = text.find("#002").expect("002");
        let pos1 = text.find("#001").expect("001");
        assert!(pos2 < pos1);
    }

    #[test]
    fn export_truncates_at_limit() {
        let big = "x".repeat(60_000);
        let notes = vec![ExportNote {
            num: 1,
            tag: "Work".into(),
            created_utc: None,
            transcript: big,
            has_text: true,
        }];
        let text = format_export_text("All", &notes);
        assert!(text.contains("Export truncated"));
    }

    #[test]
    fn export_filters_by_tag() {
        let notes = vec![
            ExportNote {
                num: 1,
                tag: "Work".into(),
                created_utc: None,
                transcript: "A".into(),
                has_text: true,
            },
            ExportNote {
                num: 2,
                tag: "Idea".into(),
                created_utc: None,
                transcript: "B".into(),
                has_text: true,
            },
        ];
        let text = format_export_text("Work", &notes);
        assert!(text.contains("#001 · Work"));
        assert!(!text.contains("#002"));
    }
}
