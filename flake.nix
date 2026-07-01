{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      rust-toolchain = pkgs.rust-bin.nightly.latest.default.override {
        extensions = [ "rust-src" ];
        targets = [
          "x86_64-unknown-linux-gnu"
        ];
      };
    in
    {
      devShells."${system}".default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rust-toolchain
          rustPlatform.bindgenHook
          cmake
          pkg-config
          openssl
          libclang
          fcitx5
          libxkbcommon
        ];
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      };
    };
}
