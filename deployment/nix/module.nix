# NixOS Module for CIM Person Domain Service
#
# Usage in configuration.nix:
#   imports = [ /path/to/cim-domain-person/deployment/nix/module.nix ];
#
#   services.cim-domain-person = {
#     enable = true;
#     natsUrl = "nats://10.0.0.41:4222";
#     streamName = "PERSON_EVENTS";
#   };

{ config, lib, pkgs, ... }:

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

    # Build only the person-service binary
    cargoBuildFlags = [ "--bin" "person-service" ];

    meta = with lib; {
      description = "CIM Person Domain Service - Event-sourced person management via NATS";
      homepage = "https://github.com/thecowboyai/cim-domain-person";
      license = licenses.mit;
      maintainers = [ ];
    };
  };

in {
  options.services.cim-domain-person = {
    enable = mkEnableOption "CIM Person Domain Service";

    package = mkOption {
      type = types.package;
      default = person-service;
      description = "The person-service package to use";
    };

    natsUrl = mkOption {
      type = types.str;
      default = "nats://localhost:4222";
      description = "NATS server URL";
      example = "nats://10.0.0.41:4222";
    };

    streamName = mkOption {
      type = types.str;
      default = "PERSON_EVENTS";
      description = "JetStream stream name for person events";
    };

    logLevel = mkOption {
      type = types.enum [ "trace" "debug" "info" "warn" "error" ];
      default = "info";
      description = "Logging level";
    };

    snapshotFrequency = mkOption {
      type = types.int;
      default = 100;
      description = "Take snapshot every N events";
    };

    user = mkOption {
      type = types.str;
      default = "cim-person";
      description = "User account under which the service runs";
    };

    group = mkOption {
      type = types.str;
      default = "cim-person";
      description = "Group account under which the service runs";
    };

    dataDir = mkOption {
      type = types.path;
      default = "/var/lib/cim-person";
      description = "Data directory for the service";
    };

    environmentFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = "Environment file with additional configuration";
      example = "/run/secrets/cim-person-env";
    };

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Open firewall for NATS communication (if needed)";
    };
  };

  config = mkIf cfg.enable {
    # Create user and group
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "CIM Person Domain Service user";
      home = cfg.dataDir;
      createHome = true;
    };

    users.groups.${cfg.group} = {};

    # Systemd service
    systemd.services.cim-domain-person = {
      description = "CIM Person Domain Service";
      documentation = [ "https://github.com/thecowboyai/cim-domain-person" ];
      after = [ "network-online.target" "nats.service" ];
      wants = [ "network-online.target" ];
      requires = [ "nats.service" ];
      wantedBy = [ "multi-user.target" ];

      environment = {
        NATS_URL = cfg.natsUrl;
        STREAM_NAME = cfg.streamName;
        LOG_LEVEL = cfg.logLevel;
        SNAPSHOT_FREQ = toString cfg.snapshotFrequency;
      };

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        WorkingDirectory = cfg.dataDir;
        ExecStart = "${cfg.package}/bin/person-service";

        # Environment file (for secrets)
        EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;

        # Security hardening
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ cfg.dataDir ];
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        SystemCallFilter = "@system-service";
        SystemCallArchitectures = "native";

        # Resource limits
        LimitNOFILE = 65536;
        TasksMax = 4096;

        # Restart policy
        Restart = "always";
        RestartSec = "10s";
        StartLimitBurst = 5;
        StartLimitIntervalSec = "60s";
      };
    };

    # Firewall configuration (if needed)
    networking.firewall = mkIf cfg.openFirewall {
      # NATS typically doesn't require incoming connections from this service
      # But you can add ports here if needed
      allowedTCPPorts = [ ];
    };
  };
}
