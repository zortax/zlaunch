# zlaunch

![GitHub License](https://img.shields.io/github/license/zortax/zlaunch)
![GitHub Release](https://img.shields.io/github/v/release/zortax/zlaunch)
![AUR Version](https://img.shields.io/aur/version/zlaunch-bin)

A fast application launcher and window switcher for Linux Wayland, built with
[GPUI](https://github.com/zed-industries/zed).

**[Website](https://zlaunch.zortax.de)** &middot; **[Documentation](https://zlaunch.zortax.de/docs)**

---

<p align="center">
<img src="https://zlaunch.zortax.de/img/hero.webp" alt="zlaunch screenshot" width="80%">
</p>

---

<details>
<summary>Demo video</summary>
<p align="center">
<video src="https://github.com/user-attachments/assets/e11cd113-798d-4b8c-84d6-36c0ff0dc3d6" width="80%" controls></video>
</p>
</details>

## Features

- **Application launching** — Fuzzy search through desktop entries with icons
- **Window switching** — Switch between open windows (Hyprland, Niri, KWin)
- **Calculator** — Evaluate math expressions and copy the result to clipboard
- **Web search** — Search Google, DuckDuckGo, Wikipedia, YouTube, and more
- **Emoji picker** — Searchable emoji grid
- **Clipboard history** — Browse and paste from clipboard history
- **AI mode** — Query local or cloud LLMs with streaming responses
- **Theming** — 15 bundled themes plus custom theme support
- **Daemon architecture** — Runs in the background for instant response

## Installation

### Arch Linux

Available on the AUR as [`zlaunch-bin`](https://aur.archlinux.org/packages/zlaunch-bin) or [`zlaunch-git`](https://aur.archlinux.org/packages/zlaunch-git).

### Nix / NixOS

```nix
{
  inputs = {
    zlaunch.url = "github:zortax/zlaunch/0.5.0";
  };
}
```

The package is available as `zlaunch.packages.${pkgs.system}.default`.

#### Home Manager module

```nix
{
  services.zlaunch = {
    enable = true;
    systemd.enabled = true;

    settings = {
      theme = "one-dark";
      launcher_size = [ 800.0 500.0 ];

      search_providers = [
        {
          name = "GitHub";
          trigger = "!gh";
          url = "https://github.com/search?q={query}";
        }
      ];
    };
  };
}
```

You can use zlaunch through a Home Manager module available as `zlaunch.homeManagerModules.default`. You can enable it and use it managed by a systemd service. To configure it use the `settings` attribute, with the same sintax as `config.toml`. Check all available options in `flake.nix`.

### Building from source

```bash
cargo build --release
```

If you have Nix installed, you can use `nix develop` for a preconfigured dev shell with all dependencies.

## Quick Start

```bash
zlaunch          # Start the daemon
zlaunch toggle   # Toggle the launcher (bind this to a key)
```

See the [documentation](https://zlaunch.zortax.de/docs) for configuration, theming, keybindings, and more.

## License

MIT
