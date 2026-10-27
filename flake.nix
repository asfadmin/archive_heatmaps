{
  description = "Rust development for Dioxus Web";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        commonTools = with pkgs; [
          trunk
          openssl
          pkg-config
          binaryen
        ];

        devToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        opensslEnv = {
          OPENSSL_NO_VENDOR = 1;
          OPENSSL_DIR = "${pkgs.openssl.out}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };
      in
      {
        devShells.default = pkgs.mkShell ({
          buildInputs = [ devToolchain ] ++ commonTools;

          shellHook = ''
            echo "Rust nightly development environment loaded."
          '';
        } //opensslEnv);
      }
    );
}
