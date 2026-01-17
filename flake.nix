{
  description = "zlaunch flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "zlaunch";
          version = "0.1.0";
          src = ./.;
          
          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper
          ];

          buildInputs = with pkgs; [
            openssl
            libxkbcommon
            xorg.libxcb
            wayland
            freetype
            fontconfig
          ];

          cargoDeps = pkgs.rustPlatform.importCargoLock {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "collections-0.1.0" = "sha256-zh+9n9h3Vu7BbFczueXs3dC5sObMEPbKJMIS9YQPqYc=";
              "xim-ctext-0.3.0" = "sha256-pRT4Sz1JU9ros47/7pmIW9kosWOGMOItcnNd+VrvnpE=";
              "zbus-5.11.0" = "sha256-MM6KukXwiU6tovU+mJODjBhJ/OiHRdC6Yp1qPG9XlR4=";
              "zed-font-kit-0.14.1-zed" = "sha256-rxpumYP0QpHW+4e+J1qo5lEZXfBk1LaL/Y0APkUp9cg=";
              "zed-reqwest-0.12.15-zed" = "sha256-p4SiUrOrbTlk/3bBrzN/mq/t+1Gzy2ot4nso6w6S+F8=";
              "zed-scap-0.0.8-zed" = "sha256-BihiQHlal/eRsktyf0GI3aSWsUCW7WcICMsC2Xvb7kw=";
            };
          };

          postFixup = ''
            wrapProgram $out/bin/zlaunch \
              --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [
                pkgs.wayland
                pkgs.libxkbcommon
                pkgs.vulkan-loader
                pkgs.xorg.libxcb
              ]}
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
            xorg.libxcb
            wayland
            freetype
            fontconfig
          ];

          env = {
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.vulkan-loader
              pkgs.xorg.libxcb
            ];
          };
        };
      }
    );
}
