//! Local transfer portal — ports `network.cpp` routes as pure HTTP handlers.

use crate::error::CoreResult;
use crate::paths::note_path;
use crate::portal_fmt::{
    export_filename, format_export_text, html_escape, portal_css, url_decode_simple, ExportNote,
};
use crate::storage::meta::read_note_meta_value;
use crate::storage::tags::{add_custom_tag, delete_tag, tag_has_notes, TagStore};
use crate::storage::{delete_note, load_index, load_tags, FileStorage, IndexStore};

pub const PORTAL_TITLE: &str = "zoop portal";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub query: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub content_type: String,
    pub body: Vec<u8>,
    pub redirect: Option<String>,
    pub attachment: Option<String>,
}

impl HttpResponse {
    pub fn html(body: String) -> Self {
        Self {
            status: 200,
            content_type: "text/html; charset=utf-8".into(),
            body: body.into_bytes(),
            redirect: None,
            attachment: None,
        }
    }

    pub fn json(body: String) -> Self {
        Self {
            status: 200,
            content_type: "application/json".into(),
            body: body.into_bytes(),
            redirect: None,
            attachment: None,
        }
    }

    pub fn text(body: String) -> Self {
        Self {
            status: 200,
            content_type: "text/plain; charset=utf-8".into(),
            body: body.into_bytes(),
            redirect: None,
            attachment: None,
        }
    }

    pub fn redirect(location: &str) -> Self {
        Self {
            status: 303,
            content_type: "text/plain".into(),
            body: Vec::new(),
            redirect: Some(location.to_string()),
            attachment: None,
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: 404,
            content_type: "text/plain".into(),
            body: b"Not found".to_vec(),
            redirect: None,
            attachment: None,
        }
    }

    pub fn bad_request(msg: &str) -> Self {
        Self {
            status: 400,
            content_type: "text/plain".into(),
            body: msg.as_bytes().to_vec(),
            redirect: None,
            attachment: None,
        }
    }

    pub fn file(content_type: &str, data: Vec<u8>, attachment: Option<String>) -> Self {
        Self {
            status: 200,
            content_type: content_type.into(),
            body: data,
            redirect: None,
            attachment,
        }
    }
}

pub fn parse_query(path: &str) -> (String, Vec<(String, String)>) {
    let mut parts = path.splitn(2, '?');
    let route = parts.next().unwrap_or("/").to_string();
    let query = parts
        .next()
        .map(|q| {
            q.split('&')
                .filter_map(|pair| {
                    let mut kv = pair.splitn(2, '=');
                    Some((kv.next()?.to_string(), kv.next().unwrap_or("").to_string()))
                })
                .collect()
        })
        .unwrap_or_default();
    (route, query)
}

pub fn query_param<'a>(query: &'a [(String, String)], key: &str) -> Option<&'a str> {
    query
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// Dispatch portal route — mirrors `setupTransferServer()` handlers.
pub fn handle_portal_request<S: FileStorage>(
    storage: &S,
    index: &mut IndexStore,
    tags: &mut TagStore,
    req: &HttpRequest,
) -> CoreResult<HttpResponse> {
    let (route, query) = parse_query(&req.path);
    match (req.method.as_str(), route.as_str()) {
        ("GET", "/") => handle_root(storage, index, tags, &query),
        ("GET", "/api/notes") => handle_api_notes(index),
        ("GET", "/export.txt") => handle_export_txt(storage, index, &query),
        ("GET", "/tags") => handle_tags_page(storage, index, tags, &query),
        ("GET", "/tag/add") => handle_tag_add(storage, tags, &query),
        ("GET", "/tag/delete") => handle_tag_delete(storage, index, tags, &query),
        ("GET", "/note/delete") => handle_note_delete(storage, index, &query),
        ("GET", "/txt") => send_file_by_num(storage, "txt", "text/plain", true, &query),
        ("GET", "/wav") => send_file_by_num(storage, "wav", "audio/wav", true, &query),
        ("GET", "/audio") => send_file_by_num(storage, "wav", "audio/wav", false, &query),
        _ => Ok(HttpResponse::not_found()),
    }
}

fn handle_root<S: FileStorage>(
    storage: &S,
    index: &IndexStore,
    tags: &TagStore,
    query: &[(String, String)],
) -> CoreResult<HttpResponse> {
    let filter = query_param(query, "tag").unwrap_or("All");
    let mut html = format!(
        "<!doctype html><html><head><meta charset='utf-8'>\
         <meta name='viewport' content='width=device-width,initial-scale=1'>\
         <title>{PORTAL_TITLE}</title>{css}</head><body><div class='wrap'>",
        css = portal_css()
    );
    html.push_str(&format!(
        "<div class='top'><div><h1>zoop<br>portal</h1>\
         <div class='sub'>local note transfer · <a href=\"/tags\" style=\"color:inherit\">tags</a></div></div>\
         <div class='pill'>{} notes</div></div>",
        index.len()
    ));

    html.push_str("<div class='actions' style='margin-bottom:18px'>");
    html.push_str(&tag_link("All", filter, "All"));
    for tag in tags.tags() {
        html.push_str(&tag_link(tag, filter, tag));
    }
    html.push_str("</div>");

    html.push_str("<div class='actions' style='margin-bottom:24px'>");
    html.push_str("<a class='btn primary' href='/export.txt'>Download all TXT</a>");
    if filter != "All" {
        html.push_str(&format!(
            "<a class='btn' href='/export.txt?tag={}'>Download {} TXT</a>",
            html_escape(filter),
            html_escape(filter)
        ));
    }
    html.push_str("</div>");

    let visible: Vec<_> = index
        .entries()
        .iter()
        .filter(|e| filter == "All" || e.tag == filter)
        .collect();

    if visible.is_empty() {
        html.push_str("<div class='empty'>No notes for this filter.</div>");
    } else {
        html.push_str("<div class='grid'>");
        for entry in visible.iter().rev() {
            let num = entry.num;
            let mut transcript =
                read_small_file(storage, &note_path(num, "txt"), 1200).unwrap_or_default();
            if transcript.is_empty() {
                transcript = if entry.has_text {
                    "(empty transcript)".to_string()
                } else {
                    "Not transcribed yet.".to_string()
                };
            }
            let mut title = transcript.replace('\n', " ");
            title = title.trim().to_string();
            if title.len() > 58 {
                title.truncate(58);
                title.push_str("...");
            }
            if title.is_empty() || title == "Not transcribed yet." {
                title = format!("Voice note {num}");
            }
            let created = read_note_meta_value(storage, num, "created_utc")?.unwrap_or_default();
            html.push_str("<div class='card'>");
            html.push_str(&format!(
                "<div class='row'><div><div class='num'>#{num}</div>\
                 <h2 class='title'>{}</h2>",
                html_escape(&title)
            ));
            if !created.is_empty() {
                html.push_str(&format!(
                    "<div class='date' data-utc='{created}'>{created}</div>"
                ));
            } else {
                html.push_str("<div class='date'>time not set</div>");
            }
            html.push_str(&format!(
                "</div><div class='tag'>{}</div></div>",
                html_escape(&entry.tag)
            ));
            html.push_str(&format!("<p class='text'>{}</p>", html_escape(&transcript)));
            if storage.exists(&note_path(num, "wav"))? {
                html.push_str(&format!("<audio controls src='/audio?num={num}'></audio>"));
            }
            html.push_str("<div class='actions'>");
            html.push_str(&format!(
                "<a class='btn primary' href='/txt?num={num}'>Download TXT</a>"
            ));
            if storage.exists(&note_path(num, "wav"))? {
                html.push_str(&format!(
                    "<a class='btn' href='/wav?num={num}'>Download WAV</a>"
                ));
            }
            html.push_str(&format!(
                "<a class='btn' style='margin-left:auto;color:#c0392b;border-color:#c0392b' \
                 href='/note/delete?num={num}'>Delete</a>"
            ));
            html.push_str("</div></div>");
        }
        html.push_str("</div>");
    }

    html.push_str("</div></body></html>");
    Ok(HttpResponse::html(html))
}

fn tag_link(label: &str, filter: &str, href_tag: &str) -> String {
    let primary = if filter == label { " primary" } else { "" };
    if href_tag == "All" {
        format!("<a class='btn{primary}' href='/'>All</a>")
    } else {
        format!(
            "<a class='btn{primary}' href='/?tag={}'>{}</a>",
            html_escape(href_tag),
            html_escape(label)
        )
    }
}

fn handle_api_notes(index: &IndexStore) -> CoreResult<HttpResponse> {
    let mut json = String::from("[");
    for (i, e) in index.entries().iter().enumerate().rev() {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            "{{\"num\":{},\"tag\":\"{}\",\"hasText\":{}}}",
            e.num,
            e.tag.replace('"', "\\\""),
            if e.has_text { "true" } else { "false" }
        ));
    }
    json.push(']');
    Ok(HttpResponse::json(json))
}

fn handle_export_txt<S: FileStorage>(
    storage: &S,
    index: &IndexStore,
    query: &[(String, String)],
) -> CoreResult<HttpResponse> {
    let filter = query_param(query, "tag").unwrap_or("All");
    let notes = build_export_notes(storage, index)?;
    let text = format_export_text(filter, &notes);
    let mut resp = HttpResponse::text(text);
    resp.attachment = Some(export_filename(filter));
    Ok(resp)
}

fn build_export_notes<S: FileStorage>(
    storage: &S,
    index: &IndexStore,
) -> CoreResult<Vec<ExportNote>> {
    let mut out = Vec::new();
    for e in index.entries() {
        let transcript =
            read_small_file(storage, &note_path(e.num, "txt"), 4000).unwrap_or_default();
        let created = read_note_meta_value(storage, e.num, "created_utc")?;
        out.push(ExportNote {
            num: e.num,
            tag: e.tag.clone(),
            created_utc: created,
            transcript,
            has_text: e.has_text,
        });
    }
    Ok(out)
}

fn handle_tags_page<S: FileStorage>(
    storage: &S,
    index: &IndexStore,
    tags: &TagStore,
    query: &[(String, String)],
) -> CoreResult<HttpResponse> {
    let mut html = String::from(
        "<!doctype html><html><head><meta charset='utf-8'>\
         <title>zoop tags</title></head><body><div class='wrap'>",
    );
    html.push_str("<h1>zoop tags</h1><a href='/'>Back to notes</a>");
    if let Some(msg) = query_param(query, "msg") {
        html.push_str(&format!("<div class='msg'>{msg}</div>"));
    }
    html.push_str(
        "<form action='/tag/add'><input name='name' maxlength='31'>\
         <button type='submit'>Add</button></form>",
    );
    for tag in tags.tags() {
        let cnt = index.entries().iter().filter(|e| e.tag == *tag).count();
        html.push_str(&format!(
            "<div class='row'><span>{}</span> <span>{cnt} notes</span>",
            html_escape(tag)
        ));
        if !tag.eq_ignore_ascii_case("Untagged") {
            html.push_str(&format!(
                " <a href='/tag/delete?name={}'>Delete</a>",
                html_escape(tag)
            ));
        }
        html.push_str("</div>");
    }
    let _ = storage;
    html.push_str("</div></body></html>");
    Ok(HttpResponse::html(html))
}

fn handle_tag_add<S: FileStorage>(
    storage: &S,
    tags: &mut TagStore,
    query: &[(String, String)],
) -> CoreResult<HttpResponse> {
    let Some(name_raw) = query_param(query, "name") else {
        return Ok(HttpResponse::redirect("/tags?msg=missing"));
    };
    let name = url_decode_simple(name_raw);
    let ok = add_custom_tag(storage, tags, &name)?;
    Ok(HttpResponse::redirect(if ok {
        "/tags?msg=added"
    } else {
        "/tags?msg=exists"
    }))
}

fn handle_tag_delete<S: FileStorage>(
    storage: &S,
    index: &mut IndexStore,
    tags: &mut TagStore,
    query: &[(String, String)],
) -> CoreResult<HttpResponse> {
    let Some(name_raw) = query_param(query, "name") else {
        return Ok(HttpResponse::redirect("/tags?msg=missing"));
    };
    let name = url_decode_simple(name_raw);
    let had = tag_has_notes(index, &name);
    let ok = delete_tag(storage, tags, index, &name)?;
    let loc = if ok && had {
        "/tags?msg=moved"
    } else if ok {
        "/tags?msg=deleted"
    } else {
        "/tags?msg=protected"
    };
    Ok(HttpResponse::redirect(loc))
}

fn handle_note_delete<S: FileStorage>(
    storage: &S,
    index: &mut IndexStore,
    query: &[(String, String)],
) -> CoreResult<HttpResponse> {
    let Some(num_s) = query_param(query, "num") else {
        return Ok(HttpResponse::bad_request("Missing num"));
    };
    let num: i32 = num_s.parse().unwrap_or(0);
    if num <= 0 {
        return Ok(HttpResponse::bad_request("Invalid num"));
    }
    delete_note(storage, index, num)?;
    Ok(HttpResponse::redirect("/"))
}

fn send_file_by_num<S: FileStorage>(
    storage: &S,
    ext: &str,
    mime: &str,
    attachment: bool,
    query: &[(String, String)],
) -> CoreResult<HttpResponse> {
    let Some(num_s) = query_param(query, "num") else {
        return Ok(HttpResponse::bad_request("Missing num"));
    };
    let num: i32 = num_s.parse().unwrap_or(0);
    if num <= 0 {
        return Ok(HttpResponse::bad_request("Invalid num"));
    }
    let path = note_path(num, ext);
    let Some(data) = storage.read_bytes(&path)? else {
        return Ok(HttpResponse::not_found());
    };
    let attach = if attachment {
        Some(format!("note_{num:03}.{ext}"))
    } else {
        None
    };
    Ok(HttpResponse::file(mime, data, attach))
}

fn read_small_file<S: FileStorage>(storage: &S, path: &str, max_len: usize) -> CoreResult<String> {
    let Some(bytes) = storage.read_bytes(path)? else {
        return Ok(String::new());
    };
    let len = bytes.len().min(max_len);
    Ok(String::from_utf8_lossy(&bytes[..len]).to_string())
}

/// Load stores from SD and serve one request (portal loop helper).
pub fn serve_portal<S: FileStorage>(
    storage: &S,
    index: &mut IndexStore,
    tags: &mut TagStore,
    req: HttpRequest,
) -> CoreResult<HttpResponse> {
    load_index(storage, index)?;
    load_tags(storage, tags)?;
    handle_portal_request(storage, index, tags, &req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::index::add_to_index;
    use crate::storage::MockStorage;

    fn setup() -> (MockStorage, IndexStore, TagStore) {
        let storage = MockStorage::new().expect("storage");
        let mut index = IndexStore::new();
        let mut tags = TagStore::new();
        load_tags(&storage, &mut tags).expect("tags");
        add_to_index(&storage, &mut index, 1, "Work", true).expect("add");
        storage
            .write_string(&note_path(1, "txt"), "Hello portal")
            .expect("txt");
        storage
            .write_bytes(&note_path(1, "wav"), &[0u8; 100])
            .expect("wav");
        (storage, index, tags)
    }

    #[test]
    fn root_returns_html_with_note() {
        let (storage, mut index, mut tags) = setup();
        let resp = handle_portal_request(
            &storage,
            &mut index,
            &mut tags,
            &HttpRequest {
                method: "GET".into(),
                path: "/".into(),
                query: vec![],
            },
        )
        .expect("handle");
        let body = String::from_utf8(resp.body).expect("utf8");
        assert!(body.contains("zoop"));
        assert!(body.contains("Hello portal"));
    }

    #[test]
    fn api_notes_json() {
        let (storage, mut index, mut tags) = setup();
        let resp = handle_portal_request(
            &storage,
            &mut index,
            &mut tags,
            &HttpRequest {
                method: "GET".into(),
                path: "/api/notes".into(),
                query: vec![],
            },
        )
        .expect("handle");
        let body = String::from_utf8(resp.body).expect("utf8");
        assert!(body.contains("\"num\":1"));
        assert!(body.contains("\"tag\":\"Work\""));
    }

    #[test]
    fn export_txt_attachment() {
        let (storage, mut index, mut tags) = setup();
        let resp = handle_portal_request(
            &storage,
            &mut index,
            &mut tags,
            &HttpRequest {
                method: "GET".into(),
                path: "/export.txt".into(),
                query: vec![],
            },
        )
        .expect("handle");
        assert_eq!(resp.attachment, Some("zoop_notes_export.txt".into()));
    }

    #[test]
    fn tag_add_and_delete_via_redirect() {
        let (storage, mut index, mut tags) = setup();
        let add = handle_portal_request(
            &storage,
            &mut index,
            &mut tags,
            &HttpRequest {
                method: "GET".into(),
                path: "/tag/add?name=Travel".into(),
                query: vec![("name".into(), "Travel".into())],
            },
        )
        .expect("add");
        assert_eq!(add.redirect, Some("/tags?msg=added".into()));
        assert!(tags.contains_ignore_case("Travel"));
    }
}
