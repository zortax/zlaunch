{
  description = "zlaunch flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.default = let
          inherit ((pkgs.lib.importTOML ./Cargo.toml).package) name version description repository;
        in
          pkgs.rustPlatform.buildRustPackage {
            pname = name;
            inherit version;

            src = self;
            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            meta = with pkgs.lib; {
              mainProgram = name;
              inherit description;
              homepage = repository;
              license = licenses.mit;
              platforms = platforms.linux;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            buildInputs = with pkgs; [
              openssl
              libxkbcommon
              libxcb
              wayland
              freetype
              fontconfig
            ];

            postFixup = with pkgs; ''
              patchelf --add-rpath ${vulkan-loader}/lib $out/bin/zlaunch
              patchelf --add-rpath ${wayland}/lib $out/bin/zlaunch
            '';
          };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            rustfmt
            clippy
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
            libxkbcommon
            libxcb
            wayland
            freetype
            fontconfig
          ];

          env = {
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.vulkan-loader
              pkgs.libxcb
            ];
          };
        };
      }
    )
    // {
      homeManagerModules.default = {
        config,
        lib,
        pkgs,
        ...
      }: let
        cfg = config.services.zlaunch;
        tomlFormat = pkgs.formats.toml {};
        pkg = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      in {
        options.services.zlaunch = {
          enable = lib.mkEnableOption "Enable zlaunch daemon";

          settings = lib.mkOption {
            type = tomlFormat.type;
            default = {};
            description = ''
              Configuration written to "~/.config/zlaunch/config.toml".
              See <https://github.com/zortax/zlaunch> for options, defined via TOML.
              All settings are optional with sensible defaults.
            '';
            example = lib.literalExpression ''
              {
                theme = "one-dark";
                launcher_size = [ 800.0 500.0 ];
                enable_backdrop = true;
                enable_transparency = true;
                hyprland_auto_blur = true;

                default_modes = ["combined" "emojis" "clipboard"];
                combined_modules = ["calculator" "windows" "applications" "actions"];

                fuzzy_match = {
                  show_best_match = true;
                };

                search_providers = [
                  {
                    name = "GitHub";
                    trigger = "!gh";
                    url = "https://github.com/search?q={query}";
                  }
                ];
              }
            '';
          };

          systemd = {
            enable = lib.mkEnableOption "Enable a zlaunch systemd service";

            autoStart = lib.mkOption {
              type = lib.types.bool;
              default = true;
              description = "Whether to automatically start the zlaunch service or not";
            };

            target = lib.mkOption {
              type = lib.types.str;
              default = "graphical-session.target";
              example = "hyprland-session.target";
              description = "Which systemd target will start the zlaunch service";
            };
          };
        };

        config = lib.mkIf cfg.enable {
          home.packages = [
            pkg
          ];

          xdg.configFile."zlaunch/config.toml" = lib.mkIf (cfg.settings != {}) {
            source = tomlFormat.generate "zlaunch-config" cfg.settings;
          };

          systemd.user.services.zlaunch = lib.mkIf cfg.systemd.enable {
            Unit = {
              Description = "Enable a zlaunch systemd service";
              Documentation = ["https://github.com/zortax/zlaunch"];
              After = [cfg.systemd.target];
              PartOf = [cfg.systemd.target];
            };
            Service = {
              Type = "simple";
              ExecStart = "${lib.getExe' pkg "zlaunch"}";
              Restart = "always";
              RestartSec = 5;
              KillMode = "process";
            };
            Install = lib.mkIf cfg.systemd.autoStart {
              WantedBy = [cfg.systemd.target];
            };
          };
        };
      };
    };
}
