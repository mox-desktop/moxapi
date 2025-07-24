{
  lib,
  config,
  pkgs,
  ...
}:
let
  cfg = config.services.moxapi;
  inherit (lib) types;
in
{
  options.services.moxapi = {
    enable = lib.mkEnableOption "moxapi";
    package = lib.mkPackageOption pkgs "moxapi" { };
    authKey = lib.mkOption { type = types.nullOr types.string; };
    authKeyFile = lib.mkOption { type = types.nullOr types.path; };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
    xdg.configFile = {
      "systemd/user/graphical-session.target.wants/moxapi.service".source =
        "${cfg.package}/share/systemd/user/moxapi.service";
    };
    systemd.user.services.moxapi = {
      environment = lib.mkForce (
        lib.optionalAttrs (cfg.authKey != null) { AUTH_KEY = cfg.authKey; }
        // lib.optionalAttrs (cfg.authKeyFile != null) { AUTH_KEY_FILE = toString cfg.authKeyFile; }
      );
    };
  };
}
