{
  description = "keyrs: keyboard remapper with Nix package and NixOS module";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ self, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem = { pkgs, config, ... }:
        let
          keyrs = pkgs.callPackage ./nix/package.nix { };
        in
        {
          packages = {
            inherit keyrs;
            default = keyrs;
          };

          apps = {
            keyrs = {
              type = "app";
              program = "${keyrs}/bin/keyrs";
              meta.description = "Run keyrs keyboard remapper";
            };
            default = config.apps.keyrs;
          };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              rustc
              cargo
              rust-analyzer
              rustfmt
              clippy
              lldb
            ];

            shellHook = ''
              echo "Rust dev shell loaded"
              echo "Rust: $(rustc --version)"
              echo "Cargo: $(cargo --version)"
            '';
          };
        };

      flake.nixosModules = {
        keyrs = import ./nix/module.nix { inherit self; };
        default = self.nixosModules.keyrs;
      };
    };
}
