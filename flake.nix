{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    { flake-parts, naersk, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.git-hooks.flakeModule
        inputs.treefmt-nix.flakeModule
      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem =
        {
          config,
          self',
          pkgs,
          ...
        }:
        {
          pre-commit = {
            check.enable = true;
            settings.hooks = {
              nixfmt-rfc-style.enable = true;
              shfmt.enable = true;
              taplo.enable = true;
              prettier = {
                enable = true;
                excludes = [ "flake.lock" ];
              };
              rustfmt.enable = true;
            };
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              shfmt.enable = true;
              taplo.enable = true;
              prettier.enable = true;
              rustfmt.enable = true;
            };
            settings.global = {
              excludes = [ ];
            };
          };

          packages.yd =
            let
              naersk' = pkgs.callPackage naersk { };
            in
            naersk'.buildPackage { src = ./.; };

          apps.yd = {
            type = "app";
            program = "${self'.packages.yd}/bin/yd";
          };

          packages.default = self'.packages.yd;
          apps.default = self'.apps.yd;

          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.pre-commit.devShell
              config.treefmt.build.devShell
            ];
            nativeBuildInputs = with pkgs; [
              pkg-config
              cargo
              rustc
            ];
            buildInputs = with pkgs; [ openssl ];
          };
        };
    };
}
