{
  description = "Framework Laptop 16 Input Module firmware and control tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "thumbv6m-none-eabi" ];
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
          # st7306 is a git dependency; fetch it via builtins.fetchGit so we
          # don't have to hand-maintain an outputHash entry.
          allowBuiltinFetchGit = true;
        };

        firmwareTarget = "thumbv6m-none-eabi";

        mkFirmware = pname:
          rustPlatform.buildRustPackage {
            inherit pname src cargoLock;
            version = "0.2.0";

            nativeBuildInputs = [ pkgs.flip-link pkgs.elf2uf2-rs ];

            # buildRustPackage's cargoBuildHook hard-codes --target to the host
            # triple, which breaks no_std firmware builds. Replace the build
            # and install phases instead of fighting the hook.
            buildPhase = ''
              runHook preBuild
              cargo build \
                --release \
                --frozen \
                --target ${firmwareTarget} \
                -p ${pname}
              runHook postBuild
            '';

            checkPhase = ''
              runHook preCheck
              runHook postCheck
            '';

            installPhase = ''
              runHook preInstall
              mkdir -p $out/bin
              cp target/${firmwareTarget}/release/${pname} $out/bin/${pname}.elf
              elf2uf2-rs \
                target/${firmwareTarget}/release/${pname} \
                $out/bin/${pname}.uf2
              runHook postInstall
            '';

            meta.description = "Framework Input Module firmware (${pname})";
          };

        inputmodule-control = rustPlatform.buildRustPackage {
          pname = "inputmodule-control";
          version = "0.2.0";
          inherit src cargoLock;

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.udev ];

          cargoBuildFlags = [ "-p" "inputmodule-control" ];
          # Override the workspace-default embedded target so this builds
          # natively for the host.
          CARGO_BUILD_TARGET = pkgs.stdenv.hostPlatform.rust.rustcTarget;

          doCheck = false;

          meta = {
            description = "Host-side control tool for Framework Input Modules";
            mainProgram = "inputmodule-control";
          };
        };
      in {
        packages = {
          default = inputmodule-control;
          inherit inputmodule-control;
          ledmatrix = mkFirmware "ledmatrix";
          b1display = mkFirmware "b1display";
          c1minimal = mkFirmware "c1minimal";
          qtpy = mkFirmware "qtpy";
        };

        apps.default = {
          type = "app";
          program = "${inputmodule-control}/bin/inputmodule-control";
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
            pkgs.flip-link
            pkgs.cargo-make
            pkgs.pkg-config
            pkgs.elf2uf2-rs
          ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.udev ];
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
