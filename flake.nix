{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      overlays = [ (import rust-overlay) ];
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs systems (
          system:
          let
            pkgs = import nixpkgs { inherit system overlays; };
          in
          function pkgs
        );
    in
    {
      devShells = forAllSystems (pkgs: {
        default =
          let
            buildInputs = [
              (pkgs.rust-bin.selectLatestNightlyWith (
                toolchain:
                toolchain.default.override {
                  extensions = [
                    "rustc-codegen-cranelift-preview"
                    "rust-src"
                    "rustfmt"
                  ];
                }
              ))
            ]
            ++ builtins.attrValues {
              inherit (pkgs)
                rust-analyzer-unwrapped
                nixd
                pkg-config
                vscode-langservers-extracted
                biome
                htmx-lsp2
                tailwindcss
                tailwindcss-language-server
                ;
            };
          in
          pkgs.mkShell {
            inherit buildInputs;
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          };
      });

      packages = forAllSystems (
        pkgs:
        let
          rustPlatform =
            let
              rust-bin = pkgs.rust-bin.stable.latest.default;
            in
            pkgs.makeRustPlatform {
              cargo = rust-bin;
              rustc = rust-bin;
            };
          node = pkgs.callPackage ./nix/node.nix { inherit rustPlatform; };
        in
        {
          inherit node;
          default = node;
          dashboard = pkgs.callPackage ./nix/dashboard.nix { inherit rustPlatform; };
        }
      );

      homeManagerModules = {
        default = import ./nix/home-manager.nix;
      };
    };
}
