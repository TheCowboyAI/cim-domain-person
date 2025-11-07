# NixOS Container Configuration for CIM Person Domain Service
#
# This module configures the person-service to run in a container environment.
# Supports:
# - Proxmox LXC containers (via nixos-generators)
# - NixOS containers (via nixos-container)
# - Scalable replicas
#
# Usage:
#   1. Proxmox LXC:
#      nix build .#person-lxc
#
#   2. NixOS container:
#      containers.person-service = {
#        autoStart = true;
#        config = import ./deployment/nix/container.nix;
#      };

{ inputs, self, ... }@flakeContext:
{ config, lib, pkgs, modulesPath, ... }:

with lib;

let
  cfg = config.services.cim-domain-person;

  # Build the person-service binary
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

    meta = with lib; {
      description = "CIM Person Domain Service";
      homepage = "https://github.com/thecowboyai/cim-domain-person";
      license = licenses.mit;
    };
  };

in {
  imports = [
    # Only import Proxmox LXC module if it exists (container environments)
    (if builtins.pathExists (modulesPath + "/virtualisation/proxmox-lxc.nix")
     then (modulesPath + "/virtualisation/proxmox-lxc.nix")
     else {})
  ];

  options.services.cim-domain-person = {
    enable = mkEnableOption "CIM Person Domain Service";

    natsUrl = mkOption {
      type = types.str;
      default = "nats://10.0.0.41:4222";
      description = "NATS server URL";
    };

    streamName = mkOption {
      type = types.str;
      default = "PERSON_EVENTS";
      description = "JetStream stream name";
    };

    logLevel = mkOption {
      type = types.enum [ "trace" "debug" "info" "warn" "error" ];
      default = "info";
      description = "Logging level";
    };

    snapshotFrequency = mkOption {
      type = types.int;
      default = 100;
      description = "Snapshot frequency in events";
    };

    containerIp = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Static IP address for container (optional)";
      example = "10.0.64.140";
    };

    gateway = mkOption {
      type = types.str;
      default = "10.0.64.1";
      description = "Default gateway";
    };

    prefixLength = mkOption {
      type = types.int;
      default = 19;
      description = "Network prefix length";
    };

    nameservers = mkOption {
      type = types.listOf types.str;
      default = [ "10.0.0.254" "1.1.1.1" ];
      description = "DNS nameservers";
    };

    sshKeys = mkOption {
      type = types.listOf types.str;
      default = [];
      description = "SSH authorized keys for root user";
      example = [ "ssh-ed25519 AAAAC3Nz... user@host" ];
    };
  };

  config = mkIf cfg.enable {
    # Container-specific settings
    boot.isContainer = mkDefault true;

    # Suppress systemd units that don't work in containers
    systemd.suppressedSystemUnits = mkDefault [
      "dev-mqueue.mount"
      "sys-kernel-debug.mount"
      "sys-fs-fuse-connections.mount"
    ];

    system.stateVersion = "24.05";

    networking = {
      hostName = "person-service";
      domain = mkDefault "cim.local";

      enableIPv6 = mkDefault false;

      # Static IP configuration (if provided)
      defaultGateway = mkIf (cfg.containerIp != null) {
        address = cfg.gateway;
        interface = "eth0";
      };

      nameservers = cfg.nameservers;

      interfaces.eth0 = mkIf (cfg.containerIp != null) {
        useDHCP = false;
        ipv4.addresses = [{
          address = cfg.containerIp;
          prefixLength = cfg.prefixLength;
        }];
      };

      firewall = {
        enable = true;
        allowedTCPPorts = [ 22 ]; # SSH only, NATS uses outbound connections
      };
    };

    # Person Domain Service
    systemd.services.cim-domain-person = {
      description = "CIM Person Domain Service";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      environment = {
        NATS_URL = cfg.natsUrl;
        STREAM_NAME = cfg.streamName;
        LOG_LEVEL = cfg.logLevel;
        SNAPSHOT_FREQ = toString cfg.snapshotFrequency;
      };

      serviceConfig = {
        Type = "simple";
        ExecStart = "${person-service}/bin/person-service";
        Restart = "always";
        RestartSec = "10s";

        # Security hardening
        DynamicUser = true;
        StateDirectory = "cim-person";

        # Filesystem protection
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;

        # Network
        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" ];

        # Capabilities
        NoNewPrivileges = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        LockPersonality = true;

        # System calls
        SystemCallFilter = "@system-service";
        SystemCallArchitectures = "native";
      };
    };

    # SSH access
    services.openssh = {
      enable = true;
      settings = {
        PasswordAuthentication = false;
        PermitRootLogin = "prohibit-password";
      };
    };

    # Root user with SSH keys
    users.users.root = mkIf (cfg.sshKeys != []) {
      openssh.authorizedKeys.keys = cfg.sshKeys;
    };

    # Essential packages
    environment.systemPackages = with pkgs; [
      htop
      tmux
      vim
      curl
      jq
    ];
  };
}
