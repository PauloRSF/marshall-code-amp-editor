# Marshall CODE Editor

A sloppy vibe-coded Marshall CODE amp settings editor. I needed this to avoid having to turn my chair 90° degrees and having to use the physical buttons and knobs ¯\_(ツ)_/¯

This is in no way endorsed or associated with the Marshall Amplification company.

## Requirements

- Rust toolchain ([rustup](https://rustup.rs/)) — stable
- A Marshall CODE connected by USB

### Windows

- Windows 10/11
- Visual Studio Build Tools (C++ workload) if you do not already have a Rust MSVC toolchain

### Linux

Install build and MIDI/UI dependencies (Debian/Ubuntu example):

```bash
sudo apt update
sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

Fedora:

```bash
sudo dnf install gcc pkgconf-pkg-config alsa-lib-devel gtk3-devel libxcb-devel libxkbcommon-devel openssl-devel
```

Your user may need permission to use MIDI devices (often membership in the `audio` group, then re-login):

```bash
sudo usermod -aG audio "$USER"
```

## Build

From the repo root:

```bash
cargo build --release
```

Binary output:

| Platform | Path |
| --- | --- |
| Windows | `target\release\MarshallCodeEditor.exe` |
| Linux | `target/release/MarshallCodeEditor` |

### Windows — Desktop install

```powershell
.\build-release.ps1
```

Builds release and copies **Marshall CODE Editor.exe** to your Desktop. You can also copy `target\release\MarshallCodeEditor.exe` anywhere and run it — no installer.

### Linux — install to `~/.local/bin`

```bash
cargo build --release
mkdir -p ~/.local/bin
cp target/release/MarshallCodeEditor ~/.local/bin/
chmod +x ~/.local/bin/MarshallCodeEditor
```

Ensure `~/.local/bin` is on your `PATH`, then run:

```bash
MarshallCodeEditor
```

## Usage

1. Power on the CODE and connect USB  
2. Launch the app and click **Connect**  
3. Edit the signal chain (ON/OFF and knobs send live MIDI)  
4. Change presets with ◀ ▶ or the preset number  
5. **Save to Amp** writes the patch into the selected slot  

**Recall** asks the amp for SysEx data. Some Windows MIDI stacks do not deliver CODE SysEx replies; live edit and save still work over USB MIDI out.

## License

MIT
