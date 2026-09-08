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

          meta = {
            mainProgram = "cp-guard";
            description = "cp-guard 是一个 Rust 实现的本地监听服务（守护进程），用于自动保存竞技编程（Competitive Programming）比赛的题目数据";
          };

          # For other makeRustPlatform features see:
          # https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md#cargo-features-cargo-features
        };

        # Used by `nix develop`
        devShells.default = pkgs.mkShell {
          buildInputs =
            with pkgs;
            [

              clippy
              rustfmt
              pkg-config
              cargo-edit
              nix-update
            ]
            ++ (with toolchain; [
              cargo
              rustc
              rustLibSrc
            ]);

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
