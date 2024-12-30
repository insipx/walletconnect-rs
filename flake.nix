# in flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
    environments.url = "github:insipx/environments";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { nixpkgs, flake-utils, fenix, environments, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ fenix.overlays.default ]; };
        inherit (pkgs.stdenv) isDarwin;
        inherit (pkgs.darwin.apple_sdk) frameworks;
        fenixPkgs = fenix.packages.${system};
        linters = import "${environments}/linters.nix" { inherit pkgs; };
        rust-toolchain = with fenixPkgs;
          combine [
            default.rustc
            default.cargo
            default.clippy
            default.rustfmt
            (complete.withComponents [ "rustc-codegen-cranelift-preview" ])
          ];
        nativeBuildInputs = with pkgs; [ pkg-config ];
        buildInputs = with pkgs;
          [
            # (fenixPkgs.fromToolchainFile { file = ./rust-toolchain.toml; })
            rust-toolchain
            rust-analyzer
            mktemp
            curl
            linters
            cargo-nextest

          ] ++ lib.optionals isDarwin [
            libiconv
            frameworks.CoreServices
            frameworks.Carbon
            frameworks.ApplicationServices
            frameworks.AppKit
          ];
      in
      with pkgs; {
        devShells.default = mkShell { inherit buildInputs nativeBuildInputs; };
      });
}
