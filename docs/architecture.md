# Zoop architecture

System-level diagrams for the Zoop voice notepad. Individual file docs highlight their component with `style ModuleName fill:#f96,stroke:#333,stroke-width:3px` in Mermaid.

File index: [README.md](README.md).

## System overview

```mermaid
flowchart TB
  subgraph Host["Host (Mac) — cargo test"]
    CORE["zoop-core<br/>app · state · storage · UI · network"]
    SIM["zoop-sim binary"]
    TESTS["unit + integration tests"]
    SIM --> CORE
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
    MIC["ES8311 mic/speaker"]
    SD["microSD FAT32"]
    BTN["REC + PWR buttons"]
    BAT["LiPo + ADC"]
  end

  subgraph Cloud["External"]
    WHISPER["Whisper / Cursor API"]
    NTP["NTP pools"]
  end

  BSP --> EPD & MIC & SD & BTN & BAT
  CORE -.->|HTTPS multipart| WHISPER
  CORE -.->|time sync| NTP
```

## Layered architecture

```mermaid
flowchart TB
  UI["display/ui · showIdle / showMenu / …"]
  APP["app::App · tick / boot / redraw"]
  SM["state::StateMachine"]
  DOMAIN["storage · record · wav · battery · sleep · sounds"]
  NET["network · portal · whisper · wifi · time"]
  IO["io traits · Display · Audio · FileStorage · Buttons"]
  BSP["firmware BSP · EpaperDisplay · Es8311Audio · SdStorage"]
  HW["Hardware"]

  UI --> APP
  APP --> SM
  APP --> DOMAIN
  APP --> NET
  APP --> IO
  DOMAIN --> IO
  NET --> IO
  BSP -.->|implements| IO
  BSP --> HW
```

## Data flow — offline record

```mermaid
sequenceDiagram
  participant User
  participant Buttons as buttons / ButtonPoller
  participant App as app::App
  participant SM as StateMachine
  participant Rec as record::RecordSession
  participant Audio as Audio trait
  participant Store as FileStorage
  participant UI as display::ui

  User->>Buttons: Hold REC ≥350 ms
  Buttons->>App: HoldRec
  App->>SM: Transition::HoldRec
  SM-->>App: Recording
  App->>UI: showRecording
  App->>Rec: start
  Rec->>Audio: start_record / read_record_chunk
  Rec->>Store: write note_NNN.wav (header rewrite)
  User->>Buttons: Release REC
  Buttons->>App: ReleaseRec
  App->>Rec: finalize
  App->>SM: RecordSuccess → TagSelect
  App->>UI: showTagSelect
  User->>Buttons: Confirm tag
  App->>Store: save_tag + index.csv
  App->>SM: TagSaved → Idle
```

## Data flow — sync / portal

```mermaid
sequenceDiagram
  participant App
  participant WiFi as wifi policy
  participant Whisper as network::whisper
  participant Portal as network::portal
  participant Store as storage
  participant API as STT API

  App->>WiFi: advance_wifi_connect
  WiFi-->>App: Connected
  App->>Whisper: transcribe_all pending notes
  Whisper->>Store: read note_NNN.wav
  Whisper->>API: POST multipart /v1/audio/transcriptions
  API-->>Whisper: {"text":"..."}
  Whisper->>Store: write note_NNN.txt + hasText
  App->>Portal: Transfer mode HTTP :80
  Portal->>Store: list / download / tags CRUD
```

## State machine

```mermaid
stateDiagram-v2
  [*] --> Idle
  Idle --> Recording: HoldRec
  Idle --> Menu: PwrSingle
  Recording --> Saved: ReleaseRec / success
  Recording --> Error: RecordFail
  Saved --> TagSelect: RecordSuccess
  TagSelect --> Idle: TagSaved
  Menu --> NoteList: OpenNotes
  Menu --> TagBrowser: OpenTags
  Menu --> Settings: OpenSettings
  Menu --> Idle: MenuBack
  TagBrowser --> NoteList: filter
  NoteList --> NoteDetail: select
  NoteDetail --> DeleteConfirm: RecLong
  NoteDetail --> NoteList: back
  DeleteConfirm --> NoteList: confirm/cancel
  Settings --> DeviceInfo: OpenDeviceInfo
  Settings --> Transfer: OpenTransfer
  Transfer --> Settings: ExitTransfer
  DeviceInfo --> Settings: back
  Error --> Idle: ErrorDismissed
```

## Crate dependency graph

```mermaid
flowchart LR
  FW["zoop-firmware"]
  CORE["zoop-core"]
  IDF["esp-idf-svc"]
  TOML["toml build-time secrets"]

  FW --> CORE
  FW --> IDF
  FW --> TOML
  CORE --> |"std only"| CORE
```

## Where to go next

| Topic | Doc |
|-------|-----|
| Host logic overview | [core/README.md](core/README.md) |
| **E-Ink UI capabilities + preview** | [core/ui-capabilities.md](core/ui-capabilities.md) |
| Firmware BSP overview | [firmware/README.md](firmware/README.md) |
| Build / CI | [config/README.md](config/README.md) |
| File index | [README.md](README.md) |

Each file page includes a **Component in architecture** Mermaid diagram with that module highlighted.
