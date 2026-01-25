# zlaunch

![GitHub License](https://img.shields.io/github/license/zortax/zlaunch)
![GitHub Release](https://img.shields.io/github/v/release/zortax/zlaunch)
![AUR Version](https://img.shields.io/aur/version/zlaunch-bin)

A fast application launcher and window switcher for Linux Wayland, built with
[GPUI](https://github.com/zed-industries/zed).

<p align="center">
<video src="https://github.com/user-attachments/assets/e11cd113-798d-4b8c-84d6-36c0ff0dc3d6" width="80%" controls></video>
</p>

## Features

- **Application launching** — Fuzzy search through desktop entries with icons
- **Window switching** — Switch between open windows (Hyprland)
- **Calculator** — Evaluate math expressions and copy the result to clipboard
- **Web search** — Search Google, DuckDuckGo, Wikipedia, YouTube, and more
- **Emoji picker** — Searchable emoji grid
- **Clipboard history** — Browse and paste from clipboard history
- **AI mode** — Query local or cloud LLMs with streaming responses
- **Theming** — 15 bundled themes plus custom theme support
- **Daemon architecture** — Runs in the background for instant response

## Compositor Support

- **Hyprland, Niri** — Window switching via IPC socket, clipboard fully supported
- **Other wlr-based compositors** — Untested, should work with clipboard history, windows switching not implemented
- **KDE / KWin** — WIP, window creation buggy, blur not supported, clipboard not working
- **GNOME** — Not supported (and not planned)

## Installation

### Arch Linux

Available on the AUR as

- [`zlaunch-bin`](https://aur.archlinux.org/packages/zlaunch-bin)
- [`zlaunch-git`](https://aur.archlinux.org/packages/zlaunch-git)

### Nix / NixOS

A nix flake is available to run zlaunch. You can test it out with,

```sh
nix shell github:zortax/zlaunch/0.4.0
zlaunch & bg # Run the daemon
zlaunch toggle
```

or add it to your system configuration,

```nix
{
  inputs = {
    zlaunch.url = "github:zortax/zlaunch/0.4.0";
  };
}
```

The package is now available as `zlaunch.packages.${pkgs.system}.default`, and can be added to your system/user packages.

### Building from source

#### Using Cargo

```bash
cargo build --release
```

#### Using Nix (devshell)

If you have Nix installed, you can enter a preconfigured development shell with all the dependencies and toolchains:

```
nix develop
cargo build --release
```

The binary will be located at:

```
./target/release/zlaunch
```

## Quick Start

Start the daemon:

```bash
zlaunch
```

Toggle the launcher (bind this to a key):

```bash
zlaunch toggle
```

## Usage

Control the daemon via CLI (use these commands in your key bindings):

```bash
zlaunch toggle  # Toggle visibility
zlaunch show    # Show launcher
zlaunch hide    # Hide launcher
zlaunch quit    # Stop daemon
zlaunch reload  # Restart daemon (useful after config updates)
```

### Modes

The launcher supports different modes that determine what content is shown. By default, the launcher opens in **combined** mode, showing all enabled modules together. You can also open specific modes directly or configure multiple modes to cycle through.

#### Opening specific modes

Use the `--modes` flag to open directly into specific mode(s):

```bash
# Open directly in emoji picker
zlaunch show --modes emojis

# Open directly in clipboard history
zlaunch toggle --modes clipboard

# Open with multiple modes (use Ctrl+Tab to switch)
zlaunch show --modes combined,emojis,clipboard
```

Available modes: `combined`, `applications`, `windows`, `emojis`, `clipboard`, `actions`, `search`, `calculator`, `ai`, `themes`

Mode aliases are supported: `apps`, `app`, `emoji`, `calc`, `action`, `theme`, `window`

#### Cycling between modes

When multiple modes are configured, use keyboard shortcuts to switch:

- `Ctrl+Tab` — Next mode
- `Ctrl+Shift+Tab` — Previous mode

Configure default modes in `config.toml` (see Configuration section).

### Theme management

Use the built-in theme selector in the UI, or via CLI:

```bash
zlaunch theme           # Show current theme
zlaunch theme list      # List available themes
zlaunch theme set NAME  # Set theme by name
```

## Keybindings

| Key                      | Action                |
| ------------------------ | --------------------- |
| `↑` / `↓`                | Navigate items        |
| `Tab` / `Shift+Tab`      | Navigate grid         |
| `Ctrl+Tab`               | Next mode             |
| `Ctrl+Shift+Tab`         | Previous mode         |
| `Enter`                  | Execute selected item |
| `Escape`                 | Back / Hide launcher  |

## Configuration

Config file location:

```
~/.config/zlaunch/config.toml
```

By default, zlaunch will not persist changes in the UI (theme) or auto-create the config file. Create the config file manually, after that in-UI theme changes will be persisted.

### Example configuration

```toml
theme = "one-dark"
window_width = 800.0
window_height = 500.0
hyprland_auto_blur = false
enable_transparency = false

# Modes to cycle through with Ctrl+Tab (optional)
default_modes = ["combined", "emojis", "clipboard"]

# Modules to show in combined view, in display order (optional)
combined_modules = ["calculator", "windows", "applications", "emojis", "clipboard", "actions", "themes", "ai", "search"]

[[search_providers]]
name = "Brave"
trigger = "!br"
url = "https://search.brave.com/search?q={query}"

[[search_providers]]
name = "YouTube"
trigger = "!yt"
url = "https://youtube.com/search?q={query}"
icon = "youtube-logo"
```

### Configuration options

- `theme` — Theme name (`default`, a bundled theme, or a custom theme file)
- `window_width` / `window_height` — Launcher window size
- `enable_transparency` — Defaults to `true`. Set to `false` to force an opaque background
- `hyprland_auto_blur` — Defaults to `true`. Attempts to apply Hyprland blur rules (WIP)
- `default_modes` — List of modes to cycle through with Ctrl+Tab. Defaults to `["combined"]`
- `combined_modules` — Ordered list of modules to include in combined view. Omit to show all modules
- `search_providers` — Custom web search providers

#### Available modules

- `calculator`
- `windows`
- `applications`
- `emojis`
- `clipboard`
- `actions`
- `themes`
- `ai`
- `search`

The order in `combined_modules` determines the display order of sections in combined view.

> **Note:** The `disabled_modules` option is deprecated. Use `combined_modules` instead to specify which modules to include (and in what order).

### Search providers

Each provider supports the following fields:

- `name` — Display name
- `trigger` — Search trigger (e.g. `!g`, `!ddg`)
- `url` — URL template containing `{query}`
- `icon` — Optional icon name; defaults to `magnifying-glass` if the field is missing, empty, or invalid

The `icon` field accepts the following kebab-case names that map to the embedded Phosphor icons:

- `power`
- `reboot`
- `moon`
- `lock`
- `sign-out`
- `smiley`
- `terminal`
- `clipboard`
- `clipboard-text`
- `file`
- `file-text`
- `file-image`
- `image`
- `magnifying-glass`
- `globe`
- `book-open`
- `youtube-logo`
- `brain`
- `palette`

Example:

```toml
[[search_providers]]
name = "YouTube"
trigger = "!yt"
url = "https://youtube.com/search?q={query}"
icon = "youtube-logo"
```

## Theming

### Bundled Themes

- `ayu-dark`
- `catppuccin-latte`
- `catppuccin-mocha`
- `dracula`
- `everforest`
- `gruvbox-dark`
- `kanagawa`
- `material`
- `monokai`
- `nord`
- `one-dark`
- `rose-pine`
- `solarized-dark`
- `synthwave`
- `tokyo-night`

### Custom Themes

Place custom themes in:

```
~/.config/zlaunch/themes/
```

Theme files use TOML format.

Supported color formats:

- Hex: `"#3fc3aa"` or `"#3fc3aa80"`
- RGBA: `{ r = 255, g = 128, b = 64, a = 255 }`
- HSLA: `{ h = 0.5, s = 0.8, l = 0.6, a = 1.0 }`

See [`assets/themes/`](https://github.com/zortax/zlaunch/tree/main/assets/themes) for examples.

### Background Blur

Because zlaunch uses a wlr layer-shell window, blur is not supported on most
compositors.

On **Hyprland**, zlaunch attempts to apply blur automatically via IPC (WIP).
This may fail on some systems.

Disable auto-blur:

```toml
hyprland_auto_blur = false
```

Manual Hyprland rules:

```
layerrule = blur on,match:class zlaunch
layerrule = blur_popups on,match:class zlaunch
layerrule = ignore_alpha 0.35,match:class zlaunch
```

## AI Mode

### Local models (Ollama)

Set the following environment variables:

- `OLLAMA_URL` — e.g. `http://127.0.0.1:11434`
- `OLLAMA_MODEL` — e.g. `llama3.2:latest`

Example:

```bash
OLLAMA_URL="http://127.0.0.1:11434" \
OLLAMA_MODEL="llama3.2:latest" \
zlaunch
```

### Cloud models

Set one of the following API keys:

- `GEMINI_API_KEY`
- `OPENAI_API_KEY`
- `OPENROUTER_API_KEY`

For OpenRouter, you can also set:

```
OPENROUTER_MODEL
```

## License

MIT
