use crate::error::{CoreError, CoreResult};
use crate::paths::{note_path, MAX_TAG_LEN};
use crate::paths::{INDEX_FILE, INDEX_TMP};

use super::FileStorage;

/// One row in `index.csv`: `num,tag,hasText`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteEntry {
    pub num: i32,
    pub tag: String,
    pub has_text: bool,
}

/// In-memory index with SD persistence — ports `notes.cpp` index functions.
#[derive(Debug, Default)]
pub struct IndexStore {
    entries: Vec<NoteEntry>,
}

impl IndexStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self) -> &[NoteEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn find(&self, num: i32) -> Option<&NoteEntry> {
        self.entries.iter().find(|e| e.num == num)
    }

    pub fn find_mut(&mut self, num: i32) -> Option<&mut NoteEntry> {
        self.entries.iter_mut().find(|e| e.num == num)
    }

    pub fn replace_tag(&mut self, old_tag: &str, new_tag: &str) {
        for entry in &mut self.entries {
            if entry.tag == old_tag {
                entry.tag = new_tag.to_string();
            }
        }
    }
}

pub fn load_index<S: FileStorage>(storage: &S, store: &mut IndexStore) -> CoreResult<()> {
    store.clear();
    let Some(content) = storage.read_to_string(INDEX_FILE)? else {
        return Ok(());
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let entry = parse_index_line(line)?;
        store.entries.push(entry);
    }
    Ok(())
}

pub fn save_index<S: FileStorage>(storage: &S, store: &IndexStore) -> CoreResult<()> {
    let mut out = String::new();
    for e in &store.entries {
        out.push_str(&format!(
            "{},{},{}\n",
            e.num,
            e.tag,
            if e.has_text { 1 } else { 0 }
        ));
    }
    storage.atomic_write_string(INDEX_FILE, INDEX_TMP, &out)
}

pub fn add_to_index<S: FileStorage>(
    storage: &S,
    store: &mut IndexStore,
    num: i32,
    tag: &str,
    has_text: bool,
) -> CoreResult<()> {
    let tag = sanitize_tag_field(tag)?;
    store.entries.push(NoteEntry { num, tag, has_text });
    save_index(storage, store)
}

pub fn update_index_has_text<S: FileStorage>(
    storage: &S,
    store: &mut IndexStore,
    num: i32,
) -> CoreResult<Option<String>> {
    let tag = store
        .find_mut(num)
        .map(|e| {
            e.has_text = true;
            e.tag.clone()
        })
        .ok_or(CoreError::NoteNotFound(num))?;
    save_index(storage, store)?;
    Ok(Some(tag))
}

pub fn delete_note<S: FileStorage>(
    storage: &S,
    store: &mut IndexStore,
    num: i32,
) -> CoreResult<()> {
    for ext in ["wav", "txt", "meta"] {
        let path = note_path(num, ext);
        if storage.exists(&path)? {
            storage.remove(&path)?;
        }
    }
    if let Some(pos) = store.entries.iter().position(|e| e.num == num) {
        store.entries.remove(pos);
        save_index(storage, store)?;
    }
    Ok(())
}

pub fn next_note_number(store: &IndexStore) -> i32 {
    store.entries.iter().map(|e| e.num).max().unwrap_or(0) + 1
}

fn parse_index_line(line: &str) -> CoreResult<NoteEntry> {
    let parts: Vec<&str> = line.splitn(3, ',').collect();
    if parts.len() != 3 {
        return Err(CoreError::InvalidIndexLine(line.to_string()));
    }
    let num: i32 = parts[0]
        .parse()
        .map_err(|_| CoreError::InvalidIndexLine(line.to_string()))?;
    let tag = sanitize_tag_field(parts[1])?;
    let has_text = parts[2].trim() == "1";
    Ok(NoteEntry { num, tag, has_text })
}

fn sanitize_tag_field(tag: &str) -> CoreResult<String> {
    let mut out = tag.trim().to_string();
    if out.len() > MAX_TAG_LEN {
        out.truncate(MAX_TAG_LEN);
    }
    if out.contains(',') {
        return Err(CoreError::InvalidTag("tag must not contain comma".into()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MockStorage;

    #[test]
    fn load_save_roundtrip() {
        let storage = MockStorage::new().expect("storage");
        let mut store = IndexStore::new();
        add_to_index(&storage, &mut store, 1, "Work", true).expect("add");
        add_to_index(&storage, &mut store, 2, "Idea", false).expect("add");

        let mut reloaded = IndexStore::new();
        load_index(&storage, &mut reloaded).expect("load");
        assert_eq!(reloaded.entries().len(), 2);
        assert_eq!(reloaded.find(1).unwrap().tag, "Work");
        assert!(reloaded.find(1).unwrap().has_text);
    }

    #[test]
    fn next_note_number_increments() {
        let mut store = IndexStore::new();
        store.entries.push(NoteEntry {
            num: 3,
            tag: "A".into(),
            has_text: false,
        });
        store.entries.push(NoteEntry {
            num: 7,
            tag: "B".into(),
            has_text: false,
        });
        assert_eq!(next_note_number(&store), 8);
    }

    #[test]
    fn delete_note_removes_files_and_row() {
        let storage = MockStorage::new().expect("storage");
        let mut store = IndexStore::new();
        add_to_index(&storage, &mut store, 5, "Note", false).expect("add");
        for ext in ["wav", "txt", "meta"] {
            storage
                .write_string(&note_path(5, ext), "x")
                .expect("write");
        }
        delete_note(&storage, &mut store, 5).expect("delete");
        assert!(store.find(5).is_none());
        for ext in ["wav", "txt", "meta"] {
            assert!(!storage.exists(&note_path(5, ext)).expect("exists"));
        }
    }

    #[test]
    fn atomic_index_write_uses_tmp() {
        let storage = MockStorage::new().expect("storage");
        let store = IndexStore::new();
        save_index(&storage, &store).expect("save empty");
        let files = storage.dump_files().expect("dump");
        assert!(files.contains_key(INDEX_FILE));
        assert!(!files.contains_key(INDEX_TMP));
    }
}
