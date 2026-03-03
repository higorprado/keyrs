{ self }:
{ lib, config, pkgs, ... }:
let
  cfg = config.services.keyrs;
  system = pkgs.stdenv.hostPlatform.system;
  defaultPackage = self.packages.${system}.keyrs;
  execArgs = [
    "${cfg.package}/bin/keyrs"
    "--config"
    cfg.configPath
  ] ++ cfg.extraArgs;
in
{
  options.services.keyrs = {
    enable = lib.mkEnableOption "keyrs keyboard remapper user service";

    package = lib.mkOption {
      type = lib.types.package;
      default = defaultPackage;
      description = "The keyrs package to run.";
    };

    configPath = lib.mkOption {
      type = lib.types.str;
      default = "%h/.config/keyrs/config.toml";
      description = "Path to keyrs config.toml passed to --config.";
    };

    extraArgs = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ "--verbose" ];
      description = "Extra command-line arguments for keyrs.";
    };

    enableUdevRules = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install udev rules required for keyboard/uinput access.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    services.udev.extraRules = lib.mkIf cfg.enableUdevRules ''
      KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput", TAG+="uaccess"
      SUBSYSTEM=="input", KERNEL=="event*", ENV{ID_INPUT_KEYBOARD}=="1", TAG+="uaccess"
    '';

    systemd.user.services.keyrs = {
      description = "keyrs keyboard remapper";
      wantedBy = [ "default.target" ];
      wants = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig = {
        Type = "simple";
        ExecStart = lib.concatStringsSep " " (map lib.escapeShellArg execArgs);
        Restart = "on-failure";
        RestartSec = 2;
      };
    };
  };
}
