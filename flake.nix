{
  description = "Rust development template";

  inputs = {
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      utils,
      ...
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        toolchain = pkgs.rustPlatform;
      in
      rec {
        # Executed by `nix build`
        packages.default = toolchain.buildRustPackage {
          pname = "cp-guard";
          version = "0.2.2";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          postInstall = ''
            rm -f $out/bin/xtask # devops, strip from release
          '';

          # For other makeRustPlatform features see:
          # https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md#cargo-features-cargo-features
        };

        # Executed by `nix run`
        apps.default = utils.lib.mkApp { drv = packages.default; };

        # Used by `nix develop`
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (with toolchain; [
              cargo
              rustc
              rustLibSrc
            ])
            clippy
            rustfmt
            pkg-config
            cargo-edit
            nix-update
          ];

          # Specify the rust-src path (many editors rely on this)
          RUST_SRC_PATH = "${toolchain.rustLibSrc}";

          shellHook = ''
            export SHELL=$(which zsh)
            if [ -f Session.vim ]; then
              exec nvim -S Session.vim
            fi
            exec zsh
          '';
        };
      }
    );
}
