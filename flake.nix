{
  description = "CIM Person Domain - Scalable NATS event-sourced service for NixOS and nix-darwin";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    darwin = {
      url = "github:LnL7/nix-darwin";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-generators, darwin, flake-utils, rust-overlay }:
    let
      flakeContext = {
        inherit (self) inputs;
        inherit self;
      };

      # System-specific outputs
      systemOutputs = flake-utils.lib.eachDefaultSystem (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          rustVersion = pkgs.rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };

          buildInputs = with pkgs; [
            openssl
            pkg-config
            protobuf
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          nativeBuildInputs = with pkgs; [
            rustVersion
            cargo-edit
            cargo-watch
          ];

          # Build person-service binary
          person-service = pkgs.rustPlatform.buildRustPackage {
            pname = "cim-domain-person";
            version = "0.8.0";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl ];

            cargoBuildFlags = [ "--bin" "person-service" ];

            meta = with pkgs.lib; {
              description = "CIM Person Domain Service - Event-sourced person management via NATS";
              homepage = "https://github.com/thecowboyai/cim-domain-person";
              license = licenses.mit;
            };
          };

        in
        {
          packages = {
            default = person-service;
            person-service = person-service;

            # Library package (for development)
            lib = pkgs.rustPlatform.buildRustPackage {
              pname = "cim-domain-person";
              version = "0.8.0";
              src = ./.;

              cargoLock = {
                lockFile = ./Cargo.lock;
              };

              inherit buildInputs nativeBuildInputs;

              checkType = "debug";
              doCheck = false;
            };
          };

          apps = {
            default = {
              type = "app";
              program = "${person-service}/bin/person-service";
            };
            person-service = {
              type = "app";
              program = "${person-service}/bin/person-service";
            };
          };

          devShells.default = pkgs.mkShell {
            inherit buildInputs nativeBuildInputs;

            shellHook = ''
              echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
              echo "CIM Person Domain - Development Environment"
              echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
              echo ""
              echo "Rust: $(rustc --version)"
              echo ""
              echo "Available commands:"
              echo "  cargo build --bin person-service  - Build service"
              echo "  cargo run --bin person-service    - Run service"
              echo "  cargo test                         - Run tests"
              echo ""
              echo "  nix build .#person-lxc            - Build Proxmox LXC"
              echo "  nix build .#person-service        - Build binary"
              echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            '';
          };
        });

    in systemOutputs // {
      # NixOS modules
      nixosModules = {
        default = import ./deployment/nix/module.nix;
        person-service = import ./deployment/nix/module.nix;
        container = import ./deployment/nix/container.nix flakeContext;
      };

      # NixOS configurations
      nixosConfigurations = {
        # Example: NixOS container
        person-container = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux";
          modules = [
            (import ./deployment/nix/container.nix flakeContext)
            {
              services.cim-domain-person = {
                enable = true;
                natsUrl = "nats://10.0.0.41:4222";
                streamName = "PERSON_EVENTS";
              };
            }
          ];
        };

        # Example: Proxmox LXC
        person-lxc = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux";
          modules = [
            (import ./deployment/nix/container.nix flakeContext)
            {
              services.cim-domain-person = {
                enable = true;
                natsUrl = "nats://10.0.0.41:4222";
                streamName = "PERSON_EVENTS";
                containerIp = "10.0.64.140";
                gateway = "10.0.64.1";
                prefixLength = 19;
                nameservers = [ "10.0.0.254" "1.1.1.1" ];
                sshKeys = [
                  "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDecTwCL7tc0mzBabAsFp1k9C3G30Nr+LIOE4MW4KWNO steele@thecowboy.ai"
                ];
              };
            }
          ];
        };
      };

      # Proxmox LXC and other formats
      packages = {
        x86_64-linux = {
          # Proxmox LXC container
          person-lxc = nixos-generators.nixosGenerate {
            system = "x86_64-linux";
            format = "proxmox-lxc";
            modules = [
              (import ./deployment/nix/container.nix flakeContext)
              {
                services.cim-domain-person = {
                  enable = true;
                  natsUrl = "nats://10.0.0.41:4222";
                  streamName = "PERSON_EVENTS";
                  containerIp = "10.0.64.140";
                  gateway = "10.0.64.1";
                  prefixLength = 19;
                  nameservers = [ "10.0.0.254" "1.1.1.1" ];
                  sshKeys = [
                    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDecTwCL7tc0mzBabAsFp1k9C3G30Nr+LIOE4MW4KWNO steele@thecowboy.ai"
                  ];
                };
              }
            ];
          };
        };

        # macOS support
        aarch64-darwin = {
          person-service-darwin =
            let
              pkgs = import nixpkgs {
                system = "aarch64-darwin";
                overlays = [ (import rust-overlay) ];
              };
            in pkgs.rustPlatform.buildRustPackage {
              pname = "cim-domain-person";
              version = "0.8.0";
              src = ./.;

              cargoLock = {
                lockFile = ./Cargo.lock;
              };

              nativeBuildInputs = with pkgs; [ pkg-config ];
              buildInputs = with pkgs; [
                openssl
                darwin.apple_sdk.frameworks.Security
                darwin.apple_sdk.frameworks.SystemConfiguration
              ];

              cargoBuildFlags = [ "--bin" "person-service" ];
            };
        };

        x86_64-darwin = {
          person-service-darwin =
            let
              pkgs = import nixpkgs {
                system = "x86_64-darwin";
                overlays = [ (import rust-overlay) ];
              };
            in pkgs.rustPlatform.buildRustPackage {
              pname = "cim-domain-person";
              version = "0.8.0";
              src = ./.;

              cargoLock = {
                lockFile = ./Cargo.lock;
              };

              nativeBuildInputs = with pkgs; [ pkg-config ];
              buildInputs = with pkgs; [
                openssl
                darwin.apple_sdk.frameworks.Security
                darwin.apple_sdk.frameworks.SystemConfiguration
              ];

              cargoBuildFlags = [ "--bin" "person-service" ];
            };
        };
      };

      # nix-darwin module
      darwinModules.default = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.services.cim-domain-person;
        in {
          options.services.cim-domain-person = {
            enable = mkEnableOption "CIM Person Domain Service";

            natsUrl = mkOption {
              type = types.str;
              default = "nats://localhost:4222";
              description = "NATS server URL";
            };

            streamName = mkOption {
              type = types.str;
              default = "PERSON_EVENTS";
              description = "JetStream stream name";
            };

            logLevel = mkOption {
              type = types.str;
              default = "info";
              description = "Logging level";
            };
          };

          config = mkIf cfg.enable {
            launchd.daemons.cim-domain-person = {
              serviceConfig = {
                ProgramArguments = [
                  "${self.packages.${pkgs.system}.person-service-darwin}/bin/person-service"
                ];
                EnvironmentVariables = {
                  NATS_URL = cfg.natsUrl;
                  STREAM_NAME = cfg.streamName;
                  LOG_LEVEL = cfg.logLevel;
                };
                KeepAlive = true;
                RunAtLoad = true;
                StandardErrorPath = "/var/log/cim-person.log";
                StandardOutPath = "/var/log/cim-person.log";
              };
            };
          };
        };
    };
}
