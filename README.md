# zlaunch

A fast cross-platform application launcher and window switcher, built with [GPUI](https://github.com/zed-industries/zed).

## Features

- **Application launching** - Fuzzy search through installed applications
- **Window switching** - Quickly switch between open windows (Linux only)
- **Daemon architecture** - Runs in background for instant response
- **Cross-platform** - Works on Linux and Windows

### Platform Support

| Feature | Linux | Windows |
|---------|-------|---------|
| Application launching | ✅ | ✅ |
| Window switching | ✅ (Hyprland, KDE/KWin) | ❌ |
| Icon support | ✅ | Limited |

## Usage

Run the daemon:
```bash
zlaunch
```

Control via CLI:
```bash
zlaunch toggle  # Toggle visibility
zlaunch show    # Show launcher
zlaunch hide    # Hide launcher
zlaunch quit    # Stop daemon
```

## Building

```bash
cargo build --release
```

## Keybindings

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate items |
| `Enter` | Launch/switch |
| `Escape` | Hide |

## License

MIT
