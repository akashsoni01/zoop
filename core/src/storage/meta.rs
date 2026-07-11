use crate::error::CoreResult;
use crate::paths::note_path;

use super::index::{add_to_index, IndexStore};
use super::FileStorage;

pub fn note_meta_path(num: i32) -> String {
    note_path(num, "meta")
}

pub fn read_note_meta_value<S: FileStorage>(
    storage: &S,
    num: i32,
    key: &str,
) -> CoreResult<Option<String>> {
    let path = note_meta_path(num);
    let Some(content) = storage.read_to_string(&path)? else {
        return Ok(None);
    };
    let prefix = format!("{key}=");
    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix(&prefix) {
            return Ok(Some(value.to_string()));
        }
    }
    Ok(None)
}

pub fn write_note_meta<S: FileStorage>(
    storage: &S,
    index: &IndexStore,
    num: i32,
    tag: Option<&str>,
    created_utc: Option<&str>,
) -> CoreResult<()> {
    let existing = read_note_meta_value(storage, num, "created_utc")?;
    let created = created_utc
        .map(str::to_string)
        .or(existing)
        .unwrap_or_default();

    let tag_str = tag.unwrap_or("");
    let has_text = index.find(num).map(|e| e.has_text).unwrap_or(false);

    let content = format!(
        "created_utc={created}\ntag={tag_str}\nsynced={}\n",
        if has_text { "1" } else { "0" }
    );
    storage.write_string(&note_meta_path(num), &content)
}

pub fn save_tag<S: FileStorage>(
    storage: &S,
    index: &mut IndexStore,
    num: i32,
    tag: &str,
) -> CoreResult<()> {
    write_note_meta(storage, index, num, Some(tag), None)?;
    add_to_index(storage, index, num, tag, false)
}

/// Convert UTC ISO `YYYY-MM-DDTHH:MM:SSZ` to local device label.
pub fn utc_to_local_device_label(utc_iso: &str, offset_min: i32) -> String {
    if utc_iso.len() < 16 {
        return "time not set".to_string();
    }
    let year: i32 = utc_iso[0..4].parse().unwrap_or(0);
    let month: i32 = utc_iso[5..7].parse().unwrap_or(1);
    let day: i32 = utc_iso[8..10].parse().unwrap_or(1);
    let hour: i32 = utc_iso[11..13].parse().unwrap_or(0);
    let minute: i32 = utc_iso[14..16].parse().unwrap_or(0);

    let epoch_utc = utc_to_epoch(year, month, day, hour, minute);
    let epoch_local = epoch_utc + (offset_min as i64) * 60;
    epoch_to_label(epoch_local)
}

fn utc_to_epoch(year: i32, month: i32, day: i32, hour: i32, minute: i32) -> i64 {
    let mut y = year;
    let mut m = month;
    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m - 3) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let epoch_days = era * 146097 + doe - 719468;
    epoch_days as i64 * 86400 + (hour as i64) * 3600 + (minute as i64) * 60
}

fn epoch_to_label(epoch: i64) -> String {
    let z = epoch / 86400 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as i64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as i32;
    let m = (mp + if mp < 10 { 3 } else { -9 }) as i32;
    let year = y + if m <= 2 { 1 } else { 0 };

    let secs = epoch.rem_euclid(86400);
    let hour = (secs / 3600) as i32;
    let minute = ((secs % 3600) / 60) as i32;
    format!("{year:04}-{m:02}-{d:02} {hour:02}:{minute:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::index::{add_to_index, IndexStore};
    use crate::storage::MockStorage;

    #[test]
    fn write_read_meta_roundtrip() {
        let storage = MockStorage::new().expect("storage");
        let mut index = IndexStore::new();
        add_to_index(&storage, &mut index, 1, "Work", false).expect("add");
        write_note_meta(
            &storage,
            &index,
            1,
            Some("Work"),
            Some("2026-07-11T12:30:00Z"),
        )
        .expect("write");
        assert_eq!(
            read_note_meta_value(&storage, 1, "tag").expect("read"),
            Some("Work".to_string())
        );
        assert_eq!(
            read_note_meta_value(&storage, 1, "created_utc").expect("read"),
            Some("2026-07-11T12:30:00Z".to_string())
        );
    }

    #[test]
    fn utc_to_local_applies_offset() {
        let label = utc_to_local_device_label("2026-07-11T10:00:00Z", 120);
        assert_eq!(label, "2026-07-11 12:00");
    }

    #[test]
    fn save_tag_writes_meta_and_index() {
        let storage = MockStorage::new().expect("storage");
        let mut index = IndexStore::new();
        save_tag(&storage, &mut index, 3, "Idea").expect("save");
        assert_eq!(index.find(3).unwrap().tag, "Idea");
        assert_eq!(
            read_note_meta_value(&storage, 3, "tag").expect("read"),
            Some("Idea".to_string())
        );
    }
}
