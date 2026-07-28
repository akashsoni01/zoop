# Example: HTTP Server

**Goal:** Serve simple web page with sensor JSON API.

**Prerequisites:** [wifi-client.md](./wifi-client.md), [communication/http.md](../communication/http.md)

---

## Rust Sketch

```rust
use picoserve::{App, AppBuilder, Router};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let stack = /* connected embassy-net Stack */;
    spawner.spawn(http_server(stack)).unwrap();
    loop { embassy_time::Timer::after_secs(3600).await; }
}

async fn http_server(stack: Stack<'static>) {
    let router = Router::new()
        .route("/", get(index_handler))
        .route("/api/temp", get(temp_handler));
    picoserve::Server::new(App::new(router)).run(stack, 80).await;
}
```

Handlers return `&'static str` or fixed buffers — no heap allocation.

See [projects/oled-dashboard.md](../projects/oled-dashboard.md).

*Next: [mqtt-client.md](./mqtt-client.md)*
