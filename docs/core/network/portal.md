# network/portal.rs

- **Path:** `core/src/network/portal.rs`
- **Purpose:** Local transfer portal — pure HTTP route handlers mirroring `setupTransferServer()`.
- **Key types / functions:**
  - `PORTAL_TITLE`, `HttpRequest`, `HttpResponse` (html/json/text/redirect/not_found/bad_request/file)
  - `parse_query`, `query_param`
  - `handle_portal_request` — routes: `/`, `/api/notes`, `/export.txt`, `/tags`, `/tag/add`, `/tag/delete`, `/note/delete`, `/txt`, `/wav`, `/audio`
  - `serve_portal` — load stores then handle one request
- **Dependencies:** `paths`, `portal_fmt`, `storage` (index/tags/meta)
- **Tests:** `cargo test -p zoop-core network::portal::tests`
- **Status:** Host-verified; `esp-idf` HTTP server HIL pending
- **Related:** [../portal_fmt.md](../portal_fmt.md), [../../firmware/network/portal.md](../../firmware/network/portal.md)
