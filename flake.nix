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
            inherit (pkgs) lib;
            buildInputs =
              [
                (pkgs.rust-bin.stable.latest.default.override {
                  extensions = [
                    "rust-src"
                    "rustfmt"
                  ];
                })
              ]
              ++ builtins.attrValues {
                inherit (pkgs)
                  rust-analyzer-unwrapped
                  nixd
                  pkg-config
                  deno
                  tailwindcss-language-server
                  vscode-langservers-extracted
                  prettierd
                  typescript-language-server
                  biome
                  nodejs_24
                  ;
              };
          in
          pkgs.mkShell {
            inherit buildInputs;
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          };
      });

      packages = forAllSystems (pkgs: {
        default = pkgs.callPackage ./nix/node.nix {
          rustPlatform =
            let
              rust-bin = pkgs.rust-bin.stable.latest.default;
            in
            pkgs.makeRustPlatform {
              cargo = rust-bin;
              rustc = rust-bin;
            };
        };
      });

      homeManagerModules = {
        default = import ./nix/home-manager.nix;
      };
    };
}
