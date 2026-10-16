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
- **📱 App Launcher & Customizable Quick Apps (Tab 3)**:
  - Built-in catalogue of popular streaming, audio, and utility apps (YouTube, Netflix, Prime Video, Disney+, Spotify, Plex, Kodi, VLC, SmartTube, Twitch).
  - **Customizable Quick App Pad**: Toggle any app (`Space`) to appear directly on the Tab 1 virtual remote quick buttons.
  - **Full App CRUD**: Add your own custom apps (`a`), edit existing apps (`e`), delete apps (`d` / `Delete`).
  - **Custom Hotkeys**: Assign any single-character shortcut key to instantly launch custom apps from the Remote tab.
  - Live fuzzy search / filter bar.
  - Custom URL & Android Intent launcher modal (`u`) to trigger any web link, app package, or intent URI.
- **⌨ TV Virtual Keyboard, IME & Keystroke Injection (Tab 4)**:
  - Type directly on your computer keyboard to inject text into any active text field or search bar on the TV.
  - **Triple Input Modes** (toggle anytime with `Tab`):
    - **Smart (IME Direct)**: Commits text directly into Android TV text fields via `RemoteImeBatchEdit` with full UTF-8 support.
    - **Keystrokes**: Injects character-by-character hardware key events (`RemoteKeyInject`).
    - **IME Batch**: Direct Android IME batch edit with active field counter synchronization.
  - **TV Global Search Overlay (`Ctrl+S`)**: Pops up Google TV Assistant / Search, types query, and submits automatically.
  - **Full Search History Management**: Re-send past queries (`↑`/`↓` and `Enter`), delete individual items (`Del` / `Ctrl+D`), or clear all history (`Ctrl+X`). Automatically persisted to config.
- **⚡ Macro Sequences & Custom Automation (Tab 5)**:
  - Automated multi-step actions (e.g. *Movie Night*: Wake TV → Delay 2s → Launch Netflix; *Bedtime Sleep*: Go Home → Wait 1s → Sleep).
  - **Full Macro Editor**: Add custom macros (`a`), edit existing macros (`e`), delete macros (`d` / `Delete`).
  - Intuitive step syntax: `key:<KEYCODE>`, `wait:<ms>`, `app:<URI>`, `text:<QUERY>`, `keys:<QUERY>`, `ime:<QUERY>`.
  - Persisted directly to `~/.config/googletv-tui/config.json`.
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
    googletv-tui text 192.168.1.100 "Stranger Things" --enter
    googletv-tui text 192.168.1.100 "Cyberpunk 2077" --mode keys
    googletv-tui list
    ```

---

## ⚡ Quick Install

The easiest way to install `googletv-tui` on any Linux machine (Ubuntu, Debian, Arch, Fedora, openSUSE, Alpine, etc.):

```bash
curl -fsSL https://raw.githubusercontent.com/Andrewx82/googletv-tui/main/install.sh | sh
```

> **What this does:** Automatically detects your architecture, downloads the latest pre-compiled binary from GitHub Releases, installs it to `/usr/local/bin` (or `~/.local/bin` if non-root), and sets executable permissions.

Once installed, simply launch:

```bash
googletv-tui
```

---

## 📥 Alternative Installation Methods

### Option 2: Direct Binary Download (Manual)

If you prefer downloading the binary manually without running a script:

```bash
# Download binary directly to ~/.local/bin (no root needed)
mkdir -p ~/.local/bin
curl -sSL https://github.com/Andrewx82/googletv-tui/releases/latest/download/googletv-tui-linux-x86_64 -o ~/.local/bin/googletv-tui
chmod +x ~/.local/bin/googletv-tui

# Or system-wide:
sudo curl -sSL https://github.com/Andrewx82/googletv-tui/releases/latest/download/googletv-tui-linux-x86_64 -o /usr/local/bin/googletv-tui
sudo chmod +x /usr/local/bin/googletv-tui
```

---

### Option 3: Download Tarball from GitHub Releases

Download the archive containing the binary, documentation, and license:

```bash
curl -sSL https://github.com/Andrewx82/googletv-tui/releases/latest/download/googletv-tui-v0.2.1-linux-x86_64.tar.gz | tar -xz
sudo mv googletv-tui /usr/local/bin/
```

Verify the installation:

```bash
googletv-tui --version
```

---

### Option 4: Install via Cargo

If you have Rust and Cargo installed:

```bash
cargo install --git https://github.com/Andrewx82/googletv-tui.git
```

---

### Option 5: Build from Source

#### 1. Install System Dependencies

Before compiling, install OpenSSL, pkg-config, and Protobuf compiler for your Linux distribution:

- **Debian / Ubuntu / Linux Mint / Pop!_OS**:
  ```bash
  sudo apt update
  sudo apt install -y build-essential libssl-dev pkg-config protobuf-compiler git
  ```
- **Arch Linux / Manjaro**:
  ```bash
  sudo pacman -S --needed base-devel openssl pkgconf protobuf git
  ```
- **Fedora / RHEL**:
  ```bash
  sudo dnf install -y gcc openssl-devel pkgconf-pkg-config protobuf-compiler git
  ```
- **openSUSE**:
  ```bash
  sudo zypper install -y gcc libopenssl-devel pkg-config protobuf-devel git
  ```
- **Alpine Linux**:
  ```bash
  apk add build-base openssl-dev pkgconf protoc git
  ```

#### 2. Clone and Build

```bash
git clone https://github.com/Andrewx82/googletv-tui.git
cd googletv-tui
cargo build --release

# Optional: Install binary to /usr/local/bin
sudo cp target/release/googletv-tui /usr/local/bin/
```

---

## 🔄 How to Update

Updating `googletv-tui` to the latest version preserves your paired devices and custom configurations:

### Method 1: Using the Installer Script (Recommended)
Simply re-run the universal installer. It detects the latest release on GitHub, automatically replaces your existing binary, and keeps your configuration intact:

```bash
curl -fsSL https://raw.githubusercontent.com/Andrewx82/googletv-tui/main/install.sh | sh
```

### Method 2: Via Cargo
If you installed via Cargo:

```bash
cargo install --git https://github.com/Andrewx82/googletv-tui.git --force
```

### Method 3: Direct Binary Download
Download the latest binary release directly and overwrite your existing installation:

```bash
# If installed in ~/.local/bin:
curl -sSL https://github.com/Andrewx82/googletv-tui/releases/latest/download/googletv-tui-linux-x86_64 -o ~/.local/bin/googletv-tui && chmod +x ~/.local/bin/googletv-tui

# If installed system-wide in /usr/local/bin:
sudo curl -sSL https://github.com/Andrewx82/googletv-tui/releases/latest/download/googletv-tui-linux-x86_64 -o /usr/local/bin/googletv-tui && sudo chmod +x /usr/local/bin/googletv-tui
```

Check your installed version anytime:
```bash
googletv-tui --version
```

---

## 🗑️ How to Uninstall

### Method 1: Automated Script Uninstall
You can cleanly uninstall the binary using the installer script with the `--uninstall` flag:

```bash
curl -fsSL https://raw.githubusercontent.com/Andrewx82/googletv-tui/main/install.sh | sh -s -- --uninstall
```

### Method 2: Manual Removal
Remove the executable binary from your binary path:

```bash
# System-wide installation:
sudo rm -f /usr/local/bin/googletv-tui

# User installation:
rm -f ~/.local/bin/googletv-tui
```

If installed using Cargo:
```bash
cargo uninstall googletv-tui
```

### Method 3: Remove Configuration & Pairing Keys (Optional)
If you also want to remove your saved TV list, custom app catalogue, macros, and cryptographic pairing certificates:

```bash
rm -rf ~/.config/googletv-tui
```

---

## ⚡ First-Time Setup & Pairing Walkthrough

1. **Network Requirement**: Make sure your Linux PC and your Google TV / Android TV are connected to the same local Wi-Fi or Ethernet network.
2. **Launch the TUI**:
   ```bash
   googletv-tui
   ```
3. **Discover TV**:
   - Press `2` to open the **Devices** tab.
   - The app will automatically scan for devices advertising `_androidtvremote2._tcp`. You can press `s` anytime to re-scan.
   - If your TV is on another subnet, press `a` to enter its IP address manually.
4. **Initiate Pairing**:
   - Select your TV in the list using `↑` / `↓` and press `p` or `Enter`.
   - Your TV screen will display a **6-character PIN code** (e.g. `3F4D12`).
5. **Enter Code**:
   - Type the 6 characters in the pop-up modal.
   - The app will automatically compute the cryptographic SHA-256 certificate handshake, submit the secret, and establish a permanent pairing.
6. **Start Controlling**:
   - Press `1` to switch back to the visual remote and control your TV!

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
| `[hotkey]` | Launch Quick App (e.g. `y`, `n`, `r`, `d`, or custom hotkey) | Configured App URI |

### Devices Tab (Tab 2)

| Key | Action |
|---|---|
| `s` | Re-scan local network via mDNS |
| `Enter` | Connect to selected device |
| `p` | Pair device (opens 6-digit PIN dialog) |
| `a` | Add manual IP address modal |
| `d` / `Delete` | Remove / forget saved device |

### Apps & Quick Launch Tab (Tab 3)

| Key | Action |
|---|---|
| `Enter` | Launch selected app on TV |
| `a` | Add custom app modal (Name, URI, Category, Hotkey, Quick App) |
| `e` | Edit selected app modal |
| `d` / `Delete` | Delete custom app |
| `Space` | Toggle app on/off Remote Control Quick Apps Pad |
| `u` | Launch direct URL or Android Intent URI dialog |
| `Esc` | Clear search filter |

### Text Input & Search Tab (Tab 4)

| Key | Action |
|---|---|
| `Enter` | Send text or selected history query to TV input field |
| `Shift+Enter` | Send text and press Enter on TV (submit search query) |
| `Ctrl+S` | Trigger TV Search overlay + type query + submit |
| `Tab` | Cycle transmission mode (Smart IME / Keystrokes / Force IME) |
| `↑` / `↓` | Select from search history |
| `Del` / `Ctrl+D` | Delete selected history item |
| `Ctrl+X` | Clear entire search history |
| `Esc` | Clear text input buffer |

### Automation Macros Tab (Tab 5)

| Key | Action |
|---|---|
| `Enter` | Execute selected macro sequence on TV |
| `a` | Add custom macro modal (Name, Description, Steps) |
| `e` | Edit selected macro modal |
| `d` / `Delete` | Delete custom macro |

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
