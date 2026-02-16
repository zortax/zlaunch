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
          ];

          buildInputs = with pkgs; [
            openssl
            libxkbcommon
            libxcb
            wayland
            freetype
            fontconfig
          ];

          cargoHash = "sha256-jTsq4Ed7REQ+dPgSXud2Frr27VqF99XFRO+v5+PjTeU=";

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
    );
}
