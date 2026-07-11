use crate::error::{CoreError, CoreResult};
use crate::paths::{DEFAULT_TAGS, MAX_TAGS, MAX_TAG_LEN, TAG_FILE, TAG_TMP};

use super::index::{save_index, IndexStore};
use super::FileStorage;

/// In-memory tag list with SD persistence — ports `notes.cpp` tag functions.
#[derive(Debug, Default)]
pub struct TagStore {
    tags: Vec<String>,
}

impl TagStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn count(&self) -> usize {
        self.tags.len()
    }

    pub fn contains_ignore_case(&self, name: &str) -> bool {
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(name))
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.tags.iter().position(|t| t == name)
    }

    pub fn index_of_ignore_case(&self, name: &str) -> Option<usize> {
        self.tags.iter().position(|t| t.eq_ignore_ascii_case(name))
    }
}

pub fn load_tags<S: FileStorage>(storage: &S, store: &mut TagStore) -> CoreResult<()> {
    store.tags.clear();
    if !storage.exists(TAG_FILE)? {
        create_default_tags(storage, store)?;
        return Ok(());
    }
    let Some(content) = storage.read_to_string(TAG_FILE)? else {
        create_default_tags(storage, store)?;
        return Ok(());
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if store.contains_ignore_case(line) {
            continue;
        }
        if store.tags.len() >= MAX_TAGS {
            break;
        }
        store.tags.push(truncate_tag(line));
    }
    if store.tags.is_empty() {
        create_default_tags(storage, store)?;
    }
    Ok(())
}

pub fn save_tags<S: FileStorage>(storage: &S, store: &TagStore) -> CoreResult<()> {
    let mut out = String::new();
    for tag in &store.tags {
        if !tag.is_empty() {
            out.push_str(tag);
            out.push('\n');
        }
    }
    storage.atomic_write_string(TAG_FILE, TAG_TMP, &out)
}

fn create_default_tags<S: FileStorage>(storage: &S, store: &mut TagStore) -> CoreResult<()> {
    store.tags.clear();
    for tag in DEFAULT_TAGS {
        if store.tags.len() >= MAX_TAGS {
            break;
        }
        store.tags.push((*tag).to_string());
    }
    save_tags(storage, store)
}

/// Sanitize and add a custom tag — returns false on duplicate or invalid input.
pub fn add_custom_tag<S: FileStorage>(
    storage: &S,
    store: &mut TagStore,
    new_tag: &str,
) -> CoreResult<bool> {
    let clean = sanitize_tag_name(new_tag)?;
    if clean.is_empty() {
        return Ok(false);
    }
    if store.tags.len() >= MAX_TAGS {
        return Err(CoreError::TagLimitReached(MAX_TAGS));
    }
    if store.contains_ignore_case(&clean) {
        return Ok(false);
    }
    store.tags.push(clean);
    save_tags(storage, store)?;
    Ok(true)
}

pub fn tag_has_notes(store: &IndexStore, tag: &str) -> bool {
    store.entries().iter().any(|e| e.tag == tag)
}

pub fn replace_tag_on_notes<S: FileStorage>(
    storage: &S,
    index: &mut IndexStore,
    old_tag: &str,
    new_tag: &str,
) -> CoreResult<()> {
    index.replace_tag(old_tag, new_tag);
    save_index(storage, index)
}

pub fn delete_tag<S: FileStorage>(
    storage: &S,
    tags: &mut TagStore,
    index: &mut IndexStore,
    tag_name: &str,
) -> CoreResult<bool> {
    if tag_name.eq_ignore_ascii_case("Untagged") {
        return Ok(false);
    }

    let had_notes = tag_has_notes(index, tag_name);
    if had_notes {
        if !tags.contains_ignore_case("Untagged") && tags.tags.len() < MAX_TAGS {
            tags.tags.push("Untagged".to_string());
        }
        replace_tag_on_notes(storage, index, tag_name, "Untagged")?;
    }

    let Some(remove_idx) = tags.index_of(tag_name) else {
        return Ok(false);
    };
    tags.tags.remove(remove_idx);

    if tags.tags.is_empty() {
        create_default_tags(storage, tags)?;
    } else {
        save_tags(storage, tags)?;
    }
    Ok(true)
}

fn sanitize_tag_name(raw: &str) -> CoreResult<String> {
    let mut s = raw.trim().to_string();
    s = s.replace(',', " ");
    s = s.replace('\n', " ");
    s = s.replace('\r', " ");
    while s.contains("  ") {
        s = s.replace("  ", " ");
    }
    s = s.trim().to_string();
    if s.len() > MAX_TAG_LEN {
        s.truncate(MAX_TAG_LEN);
    }
    Ok(s)
}

fn truncate_tag(tag: &str) -> String {
    let mut s = tag.to_string();
    if s.len() > MAX_TAG_LEN {
        s.truncate(MAX_TAG_LEN);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::index::{add_to_index, IndexStore};
    use crate::storage::MockStorage;

    #[test]
    fn default_tags_created_when_missing() {
        let storage = MockStorage::new().expect("storage");
        let mut tags = TagStore::new();
        load_tags(&storage, &mut tags).expect("load");
        assert_eq!(tags.count(), DEFAULT_TAGS.len());
        assert!(tags.contains_ignore_case("Work"));
    }

    #[test]
    fn add_custom_tag_rejects_duplicate_case_insensitive() {
        let storage = MockStorage::new().expect("storage");
        let mut tags = TagStore::new();
        load_tags(&storage, &mut tags).expect("load");
        assert!(!add_custom_tag(&storage, &mut tags, "work").expect("add"));
        assert_eq!(tags.count(), DEFAULT_TAGS.len());
    }

    #[test]
    fn add_custom_tag_strips_commas_and_newlines() {
        let storage = MockStorage::new().expect("storage");
        let mut tags = TagStore::new();
        load_tags(&storage, &mut tags).expect("load");
        assert!(add_custom_tag(&storage, &mut tags, "  Foo,Bar\n").expect("add"));
        assert!(tags.contains_ignore_case("Foo Bar"));
    }

    #[test]
    fn delete_tag_moves_notes_to_untagged() {
        let storage = MockStorage::new().expect("storage");
        let mut tags = TagStore::new();
        let mut index = IndexStore::new();
        load_tags(&storage, &mut tags).expect("load");
        add_to_index(&storage, &mut index, 1, "Work", false).expect("add");
        assert!(delete_tag(&storage, &mut tags, &mut index, "Work").expect("delete"));
        assert_eq!(index.find(1).unwrap().tag, "Untagged");
        assert!(!tags.contains_ignore_case("Work"));
    }

    #[test]
    fn cannot_delete_untagged() {
        let storage = MockStorage::new().expect("storage");
        let mut tags = TagStore::new();
        let mut index = IndexStore::new();
        load_tags(&storage, &mut tags).expect("load");
        tags.tags.push("Untagged".to_string());
        assert!(!delete_tag(&storage, &mut tags, &mut index, "Untagged").expect("delete"));
    }
}
