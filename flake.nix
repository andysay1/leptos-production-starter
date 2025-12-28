{
  description = "Leptos/Axum server-first skeleton (SSR + islands) dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.cargo-leptos
            pkgs.wasm-bindgen-cli
            pkgs.binaryen
            pkgs.nodejs_20
            pkgs.pkg-config
          ];

          shellHook = ''
            echo "Dev shell ready."
            if [ ! -d node_modules ]; then
              echo " - first time: npm ci"
            fi
            echo " - dev:   make dev"
            echo " - build: cargo leptos build --release --precompress --split"
            echo " - run:   cargo run -p server --release"
          '';
        };
      }
    );
}
