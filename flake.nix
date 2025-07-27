{
  description = "Linux Distribution Agent - A universal package management command-line tool";

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
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Use stable Rust toolchain
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" ];
        };

        # Build dependencies
        buildInputs = with pkgs; [
          openssl
          pkg-config
        ];

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "linux-distro-agent";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs;

          # Skip tests during build (can be enabled if needed)
          doCheck = false;

          meta = with pkgs.lib; {
            description = "A comprehensive command-line tool for detecting Linux distributions and providing distribution-specific package management commands";
            homepage = "https://github.com/GeneticxCln/linux-distro-agent";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.linux;
            mainProgram = "linux-distro-agent";
          };
        };

        # Development shell
        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
            # Development tools
            rust-analyzer
            rustfmt
            clippy
            cargo-watch
            cargo-edit
          ]);

          shellHook = ''
            echo "🦀 Linux Distribution Agent Development Environment"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build --release  # Build the project"
            echo "  cargo test            # Run tests"
            echo "  cargo run -- detect   # Run with detect subcommand"
            echo ""
          '';
        };

        # Apps for running the tool
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/linux-distro-agent";
        };

        # Home Manager module
        homeManagerModules.default = { config, lib, pkgs, ... }:
          with lib;
          let
            cfg = config.programs.linux-distro-agent;
          in {
            options.programs.linux-distro-agent = {
              enable = mkEnableOption "Linux Distribution Agent";
              
              package = mkOption {
                type = types.package;
                default = self.packages.${pkgs.system}.default;
                description = "The linux-distro-agent package to use";
              };
              
              enableShellIntegration = mkOption {
                type = types.bool;
                default = true;
                description = "Whether to enable shell completion";
              };
            };

            config = mkIf cfg.enable {
              home.packages = [ cfg.package ];
              
              # Enable shell completions
              programs.bash.initExtra = mkIf (cfg.enableShellIntegration && config.programs.bash.enable) ''
                source <(${cfg.package}/bin/linux-distro-agent completions bash)
              '';
              
              programs.zsh.initExtra = mkIf (cfg.enableShellIntegration && config.programs.zsh.enable) ''
                source <(${cfg.package}/bin/linux-distro-agent completions zsh)
              '';
              
              programs.fish.shellInit = mkIf (cfg.enableShellIntegration && config.programs.fish.enable) ''
                ${cfg.package}/bin/linux-distro-agent completions fish | source
              '';
            };
          };
      });
}
