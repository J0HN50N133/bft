{
  description = "Fast, interactive fuzzy tab completion for Bash";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          # Dependencies
          buildInputs = [ ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

          # Patch the script to use the absolute path of the binary
          postInstall = ''
            mkdir -p $out/share/bft
            cp scripts/bft.bash $out/share/bft/bft.bash
            substituteInPlace $out/share/bft/bft.bash \
              --replace "bft" "$out/bin/bft"
          '';

          meta = with pkgs.lib; {
            description = manifest.description;
            homepage = manifest.repository;
            license = licenses.mit;
            mainProgram = "bft";
            maintainers = [ ];
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
            # Dependencies for build
            pkg-config
            carapace
            # Development version of bft for testing tab completion
            self.packages.${system}.default
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

          # Environment variables for development
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

          # Initialize bft tab completion for development
          shellHook = ''
            source <(bft --init-script)
            echo "bft tab completion enabled for development"
          '';
        };
      }
    );
}
