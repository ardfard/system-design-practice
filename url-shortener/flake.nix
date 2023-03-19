{
  description = "url shortener in rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, naersk, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) { inherit system overlays; };
        naersk' = pkgs.callPackage naersk { };
      in
      rec {

        defaultPackage = naersk'.buildPackage {
          src = ./.;
          nativeBuildInputs = [pkgs.protobuf];
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs;[
            rnix-lsp
            protobuf
            exa
            fd
            (rust-bin.stable.latest.default.override { extensions = [ "rust-src" ]; })
            rust-analyzer
            cargo-watch
            sqlx-cli
          ];

          DATABASE_URL = "sqlite:test.db";
        };
      }
    );
}
