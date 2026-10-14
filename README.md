# Google TV Remote TUI (`googletv-tui`)

> A feature-rich, ultra-fast terminal remote control and CLI for **Google TV** and **Android TV** written in pure Rust for Linux.

Built on the **Android TV Remote Protocol v2** using mutual TLS (mTLS), Protocol Buffers, and zero-configuration mDNS local discovery.

---

## ✨ Features

- **🎮 Full ASCII / Unicode Remote Control (TUI)**:
  - Responsive visual remote control replica of the Chromecast with Google TV voice remote.
  - Interactive D-Pad: `Up`, `Down`, `Left`, `Right`, and `OK / Select`.
  - Navigation: `Back` (`Esc` / `b`), `Home` (`H`), `Power` (`p`), `Input Source` (`i`), `Settings` (`s`).
  - Media & Volume: `Play/Pause` (`Space`), `Rewind` (`[`), `Fast Forward` (`]`), `Volume Up/Down` (`+`/`-`), `Mute` (`m`), `Channel +/-`.
  - Dedicated Streaming Shortcuts: `YouTube` (`y`), `Netflix` (`n`), `Prime Video` (`r`), `Disney+` (`d`), `Spotify`.
  - Vim keybindings (`h`, `j`, `k`, `l`) and arrow keys supported.
- **🌐 Zero-Conf Discovery (mDNS)**:
  - Scans local Wi-Fi for `_androidtvremote2._tcp.local.` and `_googlecast._tcp.local.` services.
  - One-keystroke re-scanning (`s`) with live spinner and automatic IP detection.
  - Manual IP entry (`a`) for static or multi-subnet network setups.
- **🔐 Built-in Cryptographic Pairing (mTLS on Port 6467)**:
  - Automatically generates self-signed 2048-bit RSA keys and X.509 client certificates.
  - Implements the complete Polo pairing protocol with SHA256 certificate modulus & exponent cross-verification.
  - Beautiful centered 6-digit hex PIN dialog with automatic submission upon typing the 6th character.
- **📱 App Launcher & Deep-Linking (Tab 3)**:
  - Built-in catalogue of popular streaming, audio, and utility apps (YouTube, Netflix, Prime Video, Disney+, Spotify, Plex, Kodi, VLC, SmartTube, Twitch).
  - Live fuzzy search / filter bar.
  - Custom URL & Android Intent launcher modal (`u`) to trigger any web link, app package, or intent URI.
- **⌨ TV Virtual Keyboard / IME (Tab 4)**:
  - Type directly on your computer keyboard to inject text into any active text field or search bar on the TV.
  - IME batch synchronization maintaining exact field counters.
  - Search history with instant re-send.
- **⚡ Macro Sequences & Automation (Tab 5)**:
  - Automated multi-step actions (e.g. *Movie Night*: Wake TV → Delay 2s → Launch Netflix; *Bedtime Sleep*: Go Home → Wait 1s → Sleep).
  - Extensible via JSON configuration.
- **📜 Live Protocol & Activity Inspector (Tab 6)**:
  - Real-time stream of incoming and outgoing Protobuf packets, ping-pong keepalive heartbeats, power state changes, and volume adjustments.
  - Color-coded severity badges with timestamps and scroll support.
- **💻 CLI & Automation Mode**:
  - Run headless commands from bash scripts, cron, or desktop hotkeys:
    ```bash
    googletv-tui scan
    googletv-tui pair 192.168.1.100
    googletv-tui key 192.168.1.100 KEYCODE_HOME
    googletv-tui launch 192.168.1.100 "https://www.youtube.com"
    googletv-tui text 192.168.1.100 "Stranger Things"
    googletv-tui list
    ```

---

## 🚀 Getting Started

### Installation & Binary

The compiled release binary is installed at `/usr/local/bin/googletv-tui`:

```bash
googletv-tui
```

### Building from Source

```bash
cd /root/googletv-tui
cargo build --release
./target/release/googletv-tui
```

---

## ⌨ Keybinding Cheat Sheet

### Global Navigation

| Key | Action |
|---|---|
| `1` | Switch to **Remote Control** tab |
| `2` | Switch to **Devices & Discovery** tab |
| `3` | Switch to **Apps Catalogue** tab |
| `4` | Switch to **Text Input / Keyboard** tab |
| `5` | Switch to **Automation Macros** tab |
| `6` | Switch to **Protocol Logs** tab |
| `Tab` / `Shift-Tab` | Cycle focus between lists and panels |
| `?` | Open Interactive Help cheat sheet |
| `q` / `Ctrl+C` | Quit |

### Remote Control (Tab 1)

| Key | Action | Android KeyCode |
|---|---|---|
| `↑` / `k` | Directional Pad Up | `KEYCODE_DPAD_UP` |
| `↓` / `j` | Directional Pad Down | `KEYCODE_DPAD_DOWN` |
| `←` / `h` | Directional Pad Left | `KEYCODE_DPAD_LEFT` |
| `→` / `l` | Directional Pad Right | `KEYCODE_DPAD_RIGHT` |
| `Enter` | Select / OK | `KEYCODE_DPAD_CENTER` |
| `Esc` / `b` | Back | `KEYCODE_BACK` |
| `H` | Home | `KEYCODE_HOME` |
| `p` / `P` | Power Toggle / Standby | `KEYCODE_POWER` |
| `+` / `=` | Volume Up | `KEYCODE_VOLUME_UP` |
| `-` / `_` | Volume Down | `KEYCODE_VOLUME_DOWN` |
| `m` | Volume Mute | `KEYCODE_VOLUME_MUTE` |
| `Space` | Play / Pause | `KEYCODE_MEDIA_PLAY_PAUSE` |
| `[` | Rewind | `KEYCODE_MEDIA_REWIND` |
| `]` | Fast Forward | `KEYCODE_MEDIA_FAST_FORWARD` |
| `/` | Assistant / Search | `KEYCODE_SEARCH` |
| `i` | TV Input Source | `KEYCODE_TV_INPUT` |
| `s` | Settings | `KEYCODE_SETTINGS` |
| `y` | Quick Launch YouTube | App Link |
| `n` | Quick Launch Netflix | App Link |
| `r` | Quick Launch Prime Video | App Link |
| `d` | Quick Launch Disney+ | App Link |

### Devices Tab (Tab 2)

| Key | Action |
|---|---|
| `s` | Re-scan local network via mDNS |
| `Enter` | Connect to selected device |
| `p` | Pair device (opens 6-digit PIN dialog) |
| `a` | Add manual IP address modal |
| `d` / `Delete` | Remove / forget saved device |

---

## 🛠 Protocol Architecture

```
                  ┌─────────────────────────────────────────┐
                  │            googletv-tui                 │
                  └────────────┬──────────────────┬─────────┘
                               │                  │
                Port 6467 (TLS)│                  │Port 6466 (TLS)
                Polo Pairing   │                  │RemoteMessage
                               ▼                  ▼
                       ╭───────────────╮  ╭───────────────╮
                       │  Pairing PIN  │  │ Remote Keys   │
                       │  Handshake    │  │ IME Text      │
                       │  SHA256 Cert  │  │ App Launcher  │
                       │  Verification │  │ Volume/Power  │
                       ╰───────┬───────╯  ╰───────┬───────╯
                               │                  │
                               └─────────┬────────┘
                                         ▼
                                 [ Google TV ]
```

- **Polo Pairing Protocol (Port 6467)**:
  1. Client initiates mutual TLS using a self-signed RSA-2048 certificate.
  2. Both sides exchange configuration and support for 6-character hexadecimal encoding.
  3. TV renders the 6-character PIN code on display.
  4. Client computes `SHA-256(client_n || client_e || server_n || server_e || pin[2..])`.
  5. The first byte of the SHA-256 digest is validated against `pin[0..2]`.
  6. The client sends the digest secret; TV validates and returns `SecretAck`.
- **Remote Control Protocol (Port 6466)**:
  1. Uses the paired client certificate for mutual TLS.
  2. Length-delimited Protocol Buffers using standard Protobuf varint framing.
  3. Automatic ping-pong keepalive prevents timeout disconnects.
  4. Real-time updates for active app package, power state, and volume levels.

---

## 📁 Configuration File

Settings and paired devices are stored at `~/.config/googletv-tui/config.json`:

```json
{
  "active_device_ip": "192.168.1.100",
  "devices": [
    {
      "name": "Living Room TV",
      "ip": "192.168.1.100",
      "paired": true,
      "last_connected": "2026-10-13T23:00:00Z",
      "vendor": "Google",
      "model": "Chromecast with Google TV"
    }
  ],
  "custom_apps": [ ... ],
  "macros": [ ... ]
}
```

Client certificates are saved in `~/.config/googletv-tui/cert.pem` and `key.pem`.
