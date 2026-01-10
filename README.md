# zlaunch

A fast application launcher and window switcher for Linux Wayland, built with
[GPUI](https://github.com/zed-industries/zed).

<p align="center">
<video src="https://github.com/user-attachments/assets/e11cd113-798d-4b8c-84d6-36c0ff0dc3d6" width="80%" controls></video>
</p>

## Features

- **Application launching** - Fuzzy search through desktop entries with icons
- **Window switching** - Switch between open windows (Hyprland)
- **Calculator** - Evaluate math expressions, copies result to clipboard
- **Web search** - Search Google, DuckDuckGo, Wikipedia, YouTube, and more
- **Emoji picker** - Searchable emoji grid
- **Clipboard history** - Browse and paste from clipboard history
- **AI mode** - Query Gemini API with streaming responses
- **Theming** - 15 bundled themes plus custom theme support
- **Daemon architecture** - Runs in background for instant response

## Installation

### Arch Linux

Available on the AUR as [zlaunch-bin](https://aur.archlinux.org/packages/zlaunch-bin) or [zlaunch-git](https://aur.archlinux.org/packages/zlaunch-git).

### Building from source

```bash
cargo build --release
```

The binary will be at `target/release/zlaunch`.

## Usage

Start the daemon:
```bash
zlaunch
```

Control via CLI (use these commands in you key binds):
```bash
zlaunch toggle  # Toggle visibility
zlaunch show    # Show launcher
zlaunch hide    # Hide launcher
zlaunch quit    # Stop daemon
```

Theme management:

Use the theme selector in the UI, or the CLI/IPC interface:

```bash
zlaunch theme           # Show current theme
zlaunch theme list      # List available themes
zlaunch theme set NAME  # Set theme by name
```

## Keybindings

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate items |
| `Tab` / `Shift+Tab` | Navigate in grid |
| `Enter` | Execute selected item |
| `Escape` | Back / Hide launcher |

## Configuration

Config file: `~/.config/zlaunch/config.toml`

By default, zlaunch will not persist changes in the UI (theme) or auto-create
the config file. Create the config file manually, after that in-UI theme
changes will be persisted.

```toml
theme = "dracula"
window_width = 600.0
window_height = 400.0
```

## Theming

### Bundled Themes

ayu-dark, catppuccin-latte, catppuccin-mocha, dracula, everforest,
gruvbox-dark, kanagawa, material, monokai, nord, one-dark, rose-pine,
solarized-dark, synthwave, tokyo-night

### Custom Themes

Place custom theme files in `~/.config/zlaunch/themes/`. Theme files are TOML
format.

Colors can be specified as:
- Hex: `"#3fc3aa"` or `"#3fc3aa80"`
- RGBA: `{ r = 255, g = 128, b = 64, a = 255 }`
- HSLA: `{ h = 0.5, s = 0.8, l = 0.6, a = 1.0 }`

See bundled themes in `assets/themes/` for examples.

### Background Blur

As a wlr layer shell window is being used, the window blur does not work on
most compositors. ~On Hyprland, zlaunch automatically applies `layerrule`s via
the Hyprland IPC socket to enable blur. To disable this, set the following in
your config:~ _currently broken (WIP)_

```toml
hyprland_auto_blur = false
```

Set rules manually for blur support:

```
layerrule = blur on,match:class zlaunch
layerrule = blur_popups on,match:class zlaunch
layerrule = ignore_alpha 0.35,match:class zlaunch
```

## Compositor Support

- **Hyprland, Niri** - Window switching via IPC socket, clipboard fully supported
- **wlr based compositors** - untested, should work with clipboard history, windows switching not implemented
- **KDE/KWin** - WIP, window creation buggy, blur not supported, clipboard not working
- other compositors will probably not work, Gnome support not planned

## AI Mode

To enable AI mode, run the daemon with the `GEMINI_API_KEY` `OPENAI_API_KEY` or `OPENROUTER_API_KEY` env var set to an
appropriate key. Model can be chosen for OpenRouter with the `OPENROUTER_MODEL` env var.

## License

MIT
