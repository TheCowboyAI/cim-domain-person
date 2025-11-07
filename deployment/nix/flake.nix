{
  description = "CIM Person Domain Service - NixOS deployment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        person-service = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cim-domain-person";
          version = "0.8.0";

          src = ../..;

          cargoLock = {
            lockFile = ../../Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];

          cargoBuildFlags = [ "--bin" "person-service" ];

          meta = with pkgs.lib; {
            description = "CIM Person Domain Service";
            homepage = "https://github.com/thecowboyai/cim-domain-person";
            license = licenses.mit;
          };
        };

      in {
        packages = {
          default = person-service;
          person-service = person-service;
        };

        apps = {
          default = {
            type = "app";
            program = "${person-service}/bin/person-service";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            pkg-config
            openssl
            nats-server
          ];
        };
      }
    ) // {
      # NixOS module
      nixosModules.default = import ./module.nix;
    };
}
