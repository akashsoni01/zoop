# Zoop architecture

System-level diagrams for **Zoop Pay** — UPI collect on Waveshare ESP32-S3 e-Paper. Individual file docs highlight their component with `style ModuleName fill:#f96,stroke:#333,stroke-width:3px` in Mermaid.

File index: [README.md](README.md).

## System overview

```mermaid
flowchart TB
  subgraph Host["Host (Mac) — cargo test"]
    CORE["zoop-core<br/>app · state · payment · upi · UI · QR"]
    SIM["zoop-sim binary"]
    PREV["zoop-ui-preview"]
    TESTS["unit + integration tests"]
    SIM --> CORE
    PREV --> CORE
    TESTS --> CORE
  end

  subgraph Device["ESP32-S3 firmware"]
    MAIN["main.rs"]
    ENGINE["FirmwareEngine"]
    BSP["BSP adapters<br/>SD · E-Ink · ES8311 · GPIO · WiFi"]
    MAIN --> ENGINE
    ENGINE --> BSP
    ENGINE --> CORE
  end

  subgraph Hardware["Waveshare board"]
    EPD["1.54″ e-Paper 200×200"]
    MIC["ES8311 (beeps / future)"]
    SD["microSD FAT32"]
    BTN["REC + PWR buttons"]
    BAT["LiPo + ADC"]
  end

  subgraph External["External"]
    UPI["Customer UPI apps<br/>scan QR"]
    NTP["NTP pools"]
    BACKEND["Payment confirm<br/>backend TBD"]
  end

  BSP --> EPD & MIC & SD & BTN & BAT
  CORE -.->|upi://pay URI in QR| UPI
  CORE -.->|time sync| NTP
  CORE -.->|webhook / poll TBD| BACKEND
```

## Layered architecture

```mermaid
flowchart TB
  UI["display/ui · home / QR / waiting / success"]
  QR["display/qr · qrcode modules → framebuffer"]
  APP["app::App · tick / boot / redraw"]
  SM["state::StateMachine"]
  PAY["payment · MerchantProfile · PaymentRequest · ledger"]
  UPI["upi · build_upi_uri"]
  DOMAIN["battery · sleep · sounds · storage traits"]
  IO["io traits · Display · Audio · FileStorage · Buttons"]
  BSP["firmware BSP · EpaperDisplay · Es8311Audio · SdStorage"]
  HW["Hardware"]

  UI --> APP
  QR --> UI
  APP --> SM
  APP --> PAY
  PAY --> UPI
  APP --> DOMAIN
  APP --> IO
  DOMAIN --> IO
  BSP -.->|implements| IO
  BSP --> HW
```

## Data flow — UPI collect

```mermaid
sequenceDiagram
  participant User as Merchant
  participant Buttons as buttons / ButtonPoller
  participant App as app::App
  participant SM as StateMachine
  participant Pay as payment
  participant UI as display::ui
  participant QR as display::qr
  participant Cust as Customer phone

  User->>Buttons: Hold REC ≥350 ms
  Buttons->>App: idle_rec_hold / HoldRec
  App->>Pay: PaymentRequest + UPI URI
  App->>SM: Transition::HoldRec
  SM-->>App: ShowQr
  App->>UI: show_qr
  UI->>QR: draw_qr_centered(upi_uri)
  Cust->>QR: Scan e-Ink QR
  User->>Buttons: Release REC
  App->>SM: ReleaseRec → Waiting
  Note over App: Host/sim auto-confirms after ~1.5s<br/>Device: backend webhook TBD
  App->>SM: PaymentSuccess → Success
  App->>Pay: ledger.push_paid
  App->>UI: show_success
  App->>SM: PaymentDone → Idle
```

## State machine

```mermaid
stateDiagram-v2
  [*] --> Idle
  Idle --> ShowQr: HoldRec / WakeToCollect
  Idle --> Menu: PwrSingle / WakeToMenu
  ShowQr --> Waiting: ReleaseRec / PaymentPending
  ShowQr --> CancelConfirm: RecLong
  ShowQr --> Idle: MenuBack
  ShowQr --> Error: PaymentFailed
  Waiting --> Success: PaymentSuccess
  Waiting --> CancelConfirm: RecLong
  Waiting --> Error: PaymentFailed
  Success --> Idle: PaymentDone
  Menu --> ShowQr: OpenCollect
  Menu --> History: OpenHistory
  Menu --> Merchant: OpenMerchant
  Menu --> Settings: OpenSettings
  Menu --> Idle: MenuBack
  History --> HistoryDetail: RecSingle
  History --> Menu: MenuBack
  HistoryDetail --> CancelConfirm: RecLong
  HistoryDetail --> History: MenuBack
  CancelConfirm --> Idle: CancelConfirmed
  CancelConfirm --> ShowQr: CancelDismissed
  Settings --> DeviceInfo: OpenDeviceInfo
  Settings --> Menu: MenuBack
  DeviceInfo --> Settings: MenuBack
  Merchant --> Menu: ExitMerchant
  Error --> Idle: ErrorDismissed
```

## Crate dependency graph

```mermaid
flowchart LR
  FW["zoop-firmware"]
  CORE["zoop-core"]
  IDF["esp-idf-svc"]
  TOML["toml build-time secrets"]
  QRCRATE["qrcode"]

  FW --> CORE
  FW --> IDF
  FW --> TOML
  CORE --> QRCRATE
  CORE --> |"std only"| CORE
```

## Legacy modules

Voice-note helpers (`record`, `wav`, `network::whisper`, `storage` index/tags, portal) remain in-tree for reuse / future sync, but the **primary UX path is UPI collect** via `payment` + `state` + `display/ui`.

## Where to go next

| Topic | Doc |
|-------|-----|
| Host logic overview | [core/README.md](core/README.md) |
| Payment models | [core/payment.md](core/payment.md) |
| State machine | [core/state.md](core/state.md) |
| **E-Ink UI + QR** | [core/ui-capabilities.md](core/ui-capabilities.md) |
| Firmware BSP overview | [firmware/README.md](firmware/README.md) |
| Build / CI | [config/README.md](config/README.md) |
| File index | [README.md](README.md) |

Each file page includes a **Component in architecture** Mermaid diagram with that module highlighted.
