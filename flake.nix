{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
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
        inherit (pkgs) lib;
        rustToolchainFor =
          p:
          p.rust-bin.stable.latest.default.override {
            # Set the build targets supported by the toolchain,
            # wasm32-unknown-unknown is required for trunk.
            targets = [ "wasm32-unknown-unknown" ];
          };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;
        # When filtering sources, we want to allow assets other than .rs files
        unfilteredRoot = ./.; # The original, unfiltered source
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            # Example of a folder for images, icons, etc
            # (lib.fileset.maybeMisssing ./assets)
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = [
            pkgs.cacert
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.openssl
            # Additional darwin specific inputs can be set here
          ] ++ lib.optionals pkgs.stdenv.isLinux [
            pkgs.openssl
            pkgs.pkg-config
          ];
          # Set SSL certificate environment variables
          SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
        };
        nativeArgs = commonArgs // {
          pname = "acli";
        };
        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly nativeArgs;
        # build the library / shared rust crate that can be
        # published to crates.io
        alib = craneLib.buildPackage (
          nativeArgs
          // {
            inherit cargoArtifacts;
          }
        );
        acli = craneLib.buildPackage (
          nativeArgs // { inherit cargoArtifacts; }
        );
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit alib acli;
          # check the docs
          docs = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
        };

        packages.default = acli;
        packages.acli = acli;
        # app to copy all outpaths from omnix result to local ./artifacts folder
        apps.collect-build-artifacts = flake-utils.lib.mkApp {
          name = "collect-build-artifacts";
          drv = pkgs.writeShellScriptBin "collect-build-artifacts" ''
            set -euo pipefail
            rm -rf artifacts
            mkdir -p artifacts
            jq -r '.result.ROOT.build.byName | to_entries[] | "\(.key):\(.value)"' result | while IFS=':' read -r name path; do
              [ -e "$path" ] && mkdir -p "artifacts/$name" && cp -r "$path" "artifacts/$name/"
            done
          '';
        };
        # app to update repository statistics, coverage info, etc
        # apps.update-repo-info = flake-utils.lib.mkApp {
        #   name = "update-repo-info";
        #   drv = pkgs.writeShellScriptBin "update-repo-info" ''
        #     set -euo pipefail
        #     echo "" > COVERAGE.md
        #     echo "# Project Information and Code Coverage" >> COVERAGE.md
        #     echo "## Code Statistics" >> COVERAGE.md
        #     nix develop -c tokei --hidden -C >> COVERAGE.md
        #   '';
        # };
        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};
          shellHook = ''
            export CLIENT_DIST=$PWD/client/dist;
            # Ensure rust-analyzer can find the toolchain
            # export RUST_SRC_PATH="${rustToolchainFor pkgs}/lib/rustlib/src/rust/library";
          '';
          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            pkgs.act
            pkgs.rust-analyzer
            pkgs.rustup
            pkgs.tokei
            pkgs.omnix
          ];
        };
      }
    );
}
