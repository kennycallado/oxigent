{
  description = "rust-kanban";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";

    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        rust-kanban = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = "rust-kanban";
          }
        );
      in
      {
        checks = {
          inherit rust-kanban;
          rust-kanban-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
          rust-kanban-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
          rust-kanban-fmt = craneLib.cargoFmt {
            inherit src;
          };
          rust-kanban-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            }
          );
        };

        packages.default = rust-kanban;

        apps.default = flake-utils.lib.mkApp {
          drv = rust-kanban;
        };

        devShells.default = craneLib.devShell {
          inputsFrom = [ rust-kanban ];
          packages = with pkgs; [
            rustToolchain
            bacon
            bun
            cargo-edit
            cargo-watch
            librsvg
            nodejs_22
            pkg-config
            pnpm
            sccache
            webkitgtk_4_1
            wrapGAppsHook4
          ];
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          RUSTC_WRAPPER = "sccache";
          SCCACHE_DIR = "$HOME/.cache/sccache";
        };
      }
    );
}
