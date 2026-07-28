# Lesson 21 — Bluetooth Low Energy (BLE, GATT)

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [20-wifi.md](./20-wifi.md) (coexistence on ESP)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md) (MCU peripheral), **CoreBluetooth** (host Swift on iOS/macOS).

**Maturity note:** MCU BLE on ESP32-S3 via **Embedded Swift + ESP-IDF NimBLE/Bluedroid** is the practical path. **CoreBluetooth on iOS/macOS** is production-ready for central/peripheral host apps that talk to your board. Bare-metal Swift BLE stack: **not available** — radio controller is closed firmware.

---

## Theory

**Bluetooth Low Energy (BLE)** is a short-range, low-power protocol. Unlike Classic Bluetooth, BLE emphasizes **GATT** (Generic Attribute Profile) data exchange.

### Roles

| Role | Behavior |
|------|----------|
| **Peripheral** | Advertises, accepts connections — your sensor |
| **Central** | Scans, connects — phone, Mac |
| **Broadcaster / Observer** | Connectionless beacons |

Most embedded firmware implements **peripheral** mode.

### GATT hierarchy

```
GATT Server (your device)
  └── Service (e.g., Nordic UART Service)
        └── Characteristic (RX — write from phone)
        └── Characteristic (TX — notify to phone)
              └── Descriptor (CCCD)
```

| Term | Meaning |
|------|---------|
| **UUID** | 128-bit ID — 16-bit shortcuts for standard services |
| **Notify** | Server pushes without ACK |
| **Indicate** | Server pushes with ACK |
| **CCCD** | Client enables notify (`0x0001`) / indicate (`0x0002`) |

### Nordic UART Service (NUS)

| UUID suffix | Characteristic | Direction |
|-------------|----------------|-----------|
| `6E400001-...` | Service | — |
| `6E400002-...` | RX | Central writes → device |
| `6E400003-...` | TX | Device notifies → central |

---

## Hardware Overview

### ESP32-S3 BLE

- Bluetooth 5.0 LE
- Shared radio with Wi-Fi — controller arbitrates
- Rust/Swift path: **esp-idf-svc** equivalent via ESP-IDF C BLE APIs

### nRF52840

- BLE-native — best for learning GATT without Wi-Fi — [boards/nrf52.md](./boards/nrf52.md)

---

## ASCII Wiring

No external wiring for BLE — antenna on-module:

```
┌─────────────────────┐
│  ESP32-S3 DevKitC-1 │
│  [ PCB antenna ]    │  ← keep away from metal
└─────────────────────┘

Phone (Central) ~~~~ BLE ~~~~►  MCU (Peripheral)
```

Optional: **nRF52840 DK** for BLE-only development with fewer coexistence concerns.

---

## Memory & Register Notes

BLE stack RAM usage is significant on ESP32 (~40–80 KB depending on config). GATT table lives in flash + RAM:

```swift
struct GATTCharacteristic {
    let uuid: UUID
    let properties: GATTProperties  // read, write, notify
    var value: [UInt8]
}

struct GATTService {
    let uuid: UUID
    var characteristics: [GATTCharacteristic]
}
```

---

## MCU Embedded Swift — BLE Peripheral (ESP-IDF)

```swift
import ESPIDF

let nusServiceUUID = UUID(uuidString: "6E400001-B5A3-F393-E0A9-E50E24DCCA9E")!
let nusRxUUID = UUID(uuidString: "6E400002-B5A3-F393-E0A9-E50E24DCCA9E")!
let nusTxUUID = UUID(uuidString: "6E400003-B5A3-F393-E0A9-E50E24DCCA9E")!

@main
struct BlePeripheralApp {
    static func main() {
        ESPBLE.init()

        var txValue: [UInt8] = []

        let server = GATTServer(
            deviceName: "SwiftSensor",
            services: [
                GATTService(uuid: nusServiceUUID, characteristics: [
                    GATTCharacteristic(uuid: nusRxUUID,
                        properties: [.write], value: []),
                    GATTCharacteristic(uuid: nusTxUUID,
                        properties: [.notify], value: txValue),
                ])
            ]
        )

        server.onWrite(uuid: nusRxUUID) { data in
            // Echo received bytes via notify
            server.notify(uuid: nusTxUUID, data: data)
        }

        server.startAdvertising()
        print("BLE advertising as SwiftSensor")

        while true {
            delay(ms: 1000)
            // Periodic sensor notify example
            let temp: [UInt8] = [0x18, 0x00] // 24 °C placeholder
            server.notify(uuid: nusTxUUID, data: temp)
        }
    }
}
```

Wire to ESP-IDF `esp_ble_gatts_*` or NimBLE APIs.

---

## Bare-Metal Swift Sketch

**Not applicable** — BLE controller firmware is closed-source on ESP32. Use ESP-IDF or nRF Connect SDK (C) with Swift app layer.

Illustrative GATT state only:

```swift
enum BleState {
    case idle
    case advertising
    case connected(handle: UInt16)
    case disconnected
}
```

---

## Host Swift — CoreBluetooth Central (iOS/macOS)

Production-ready pattern: iPhone app talks to your MCU peripheral.

```swift
import CoreBluetooth

class BleCentralManager: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate {
    private var central: CBCentralManager!
    private var peripheral: CBPeripheral?
    private var txCharacteristic: CBCharacteristic?

    let nusServiceUUID = CBUUID(string: "6E400001-B5A3-F393-E0A9-E50E24DCCA9E")
    let nusTxUUID = CBUUID(string: "6E400003-B5A3-F393-E0A9-E50E24DCCA9E")
    let nusRxUUID = CBUUID(string: "6E400002-B5A3-F393-E0A9-E50E24DCCA9E")

    override init() {
        super.init()
        central = CBCentralManager(delegate: self, queue: .main)
    }

    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        guard central.state == .poweredOn else { return }
        central.scanForPeripherals(withServices: [nusServiceUUID])
    }

    func centralManager(_ central: CBCentralManager,
                        didDiscover peripheral: CBPeripheral,
                        advertisementData: [String: Any],
                        rssi: NSNumber) {
        self.peripheral = peripheral
        peripheral.delegate = self
        central.connect(peripheral)
    }

    func centralManager(_ central: CBCentralManager,
                        didConnect peripheral: CBPeripheral) {
        peripheral.discoverServices([nusServiceUUID])
    }

    func peripheral(_ peripheral: CBPeripheral,
                    didDiscoverCharacteristicsFor service: CBService,
                    error: Error?) {
        for char in service.characteristics ?? [] {
            if char.uuid == nusTxUUID {
                txCharacteristic = char
                peripheral.setNotifyValue(true, for: char)
            }
        }
    }

    func peripheral(_ peripheral: CBPeripheral,
                    didUpdateValueFor characteristic: CBCharacteristic,
                    error: Error?) {
        guard let data = characteristic.value else { return }
        print("MCU → Phone: \(data.map { String(format: "%02x", $0) }.joined())")
    }

    func sendCommand(_ text: String) {
        guard let peripheral, let rx = peripheral.services?
            .flatMap({ $0.characteristics ?? [] })
            .first(where: { $0.uuid == nusRxUUID }) else { return }
        peripheral.writeValue(text.data(using: .utf8)!, for: rx, type: .withResponse)
    }
}
```

Add **NSBluetoothAlwaysUsageDescription** to Info.plist on iOS.

---

## Host Swift — CoreBluetooth Peripheral (Mac as sensor simulator)

Test your iOS central app without hardware:

```swift
import CoreBluetooth

class BlePeripheralSimulator: NSObject, CBPeripheralManagerDelegate {
    private var manager: CBPeripheralManager!
    private var txChar: CBMutableCharacteristic!

    func start() {
        manager = CBPeripheralManager(delegate: self, queue: .main)
    }

    func peripheralManagerDidUpdateState(_ peripheral: CBPeripheralManager) {
        guard peripheral.state == .poweredOn else { return }

        txChar = CBMutableCharacteristic(
            type: CBUUID(string: "6E400003-B5A3-F393-E0A9-E50E24DCCA9E"),
            properties: [.notify],
            value: nil,
            permissions: [.readable]
        )

        let service = CBMutableService(
            type: CBUUID(string: "6E400001-B5A3-F393-E0A9-E50E24DCCA9E"),
            primary: true
        )
        service.characteristics = [txChar]
        manager.add(service)
        manager.startAdvertising([
            CBAdvertisementDataServiceUUIDsKey: [service.uuid],
            CBAdvertisementDataLocalNameKey: "MacSimulator"
        ])
    }

    func notifyTemperature(_ celsius: UInt8) {
        manager.updateValue(Data([celsius]), for: txChar, onSubscribedCentrals: nil)
    }
}
```

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| CCCD not enabled | No notifications received | Central must write CCCD |
| UUID mismatch | Service not discovered | Match 128-bit UUIDs exactly |
| MTU too small | Truncated payloads | Negotiate MTU (ESP-IDF / CoreBluetooth) |
| Wi-Fi + BLE coexistence | Connection drops | Use Espressif coexistence config |
| iOS background BLE | App suspended | Use background modes + state restoration |
| Advertising too slow | Hard to find device | Reduce interval for bring-up |

---

## Debugging Tips

- **nRF Connect** app (iOS/Android) — scan, connect, inspect GATT.
- **LightBlue Explorer** — similar GATT browser.
- Log HCI events on ESP-IDF (`CONFIG_BT_LOG_HCI_ENABLED`).
- macOS **PacketLogger** (Apple developer tools) for host-side traces.
- Compare against Nordic UART example firmware on same phone.

---

## Performance Tips

- **Notify** over **Indicate** when ACK not required.
- Batch sensor readings — don't notify every millisecond.
- Use **connection interval** negotiation for power — [24-low-power.md](./24-low-power.md).
- Keep GATT table small — fewer characteristics = faster discovery.

---

## Exercises

1. **Advertise:** MCU peripheral named `SwiftSensor`; find with nRF Connect.
2. **NUS echo:** Phone writes string; MCU notifies echo back.
3. **CoreBluetooth central:** iOS app displays live temperature notifies.
4. **Mac simulator:** Test iOS app against Mac peripheral before hardware ready.
5. **Provisioning:** BLE NUS sends Wi-Fi credentials to MCU — [20-wifi.md](./20-wifi.md).

---

## References

- [ESP-IDF BLE API](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/bluetooth/index.html)
- [Apple CoreBluetooth documentation](https://developer.apple.com/documentation/corebluetooth)
- [Nordic UART Service spec](https://developer.nordicsemi.com/nRF_Connect_SDK/doc/latest/nrf/libraries/bluetooth/services/nus.html)
- [boards/esp32-s3.md](./boards/esp32-s3.md)
- [boards/nrf52.md](./boards/nrf52.md)

---

*Previous: [20-wifi.md](./20-wifi.md) · Next: [22-swift-concurrency.md](./22-swift-concurrency.md)*
