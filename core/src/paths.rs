//! Storage paths and limits — mirrors `config.h`.

pub const NOTES_DIR: &str = "notes";
pub const INDEX_FILE: &str = "notes/index.csv";
pub const INDEX_TMP: &str = "notes/index.tmp";
pub const TAG_FILE: &str = "notes/tags.txt";
pub const TAG_TMP: &str = "notes/tags.tmp";

pub const MAX_TAGS: usize = 20;
pub const MAX_TAG_LEN: usize = 31;

pub const DEFAULT_TAGS: &[&str] = &["Note", "Work", "Idea", "Buy", "Private"];

/// Build path for a note file: `notes/note_NNN.ext`
pub fn note_path(num: i32, ext: &str) -> String {
    format!("{NOTES_DIR}/note_{num:03}.{ext}")
}
