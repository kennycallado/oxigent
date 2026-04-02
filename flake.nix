{
  description = "oxigent";

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
          pname = "oxigent";
          strictDeps = true;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        oxigent = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = "oxigent";
          }
        );
      in
      {
        checks = {
          inherit oxigent;
          oxigent-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
          oxigent-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
          oxigent-fmt = craneLib.cargoFmt {
            inherit src;
          };
          oxigent-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            }
          );
        };

        packages.default = oxigent;

        apps.default = flake-utils.lib.mkApp {
          drv = oxigent;
        };

        devShells.default = craneLib.devShell {
          inputsFrom = [ oxigent ];
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
