{ lib, ... }:
let
  cfg = config.services.moxapi;
in
{
  options.services.moxapi = {
    enable = lib.mkEnableOption "moxapi";
    package = lib.mkPackageOption pkgs "moxapi" { };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
    xdg.configFile = {
      "systemd/user/graphical-session.target.wants/moxapi.service".source =
        "${cfg.package}/share/systemd/user/moxapi.service";
    };
  };
}
