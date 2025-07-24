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
    authKey = lib.mkOption {
      type = types.nullOr types.str;
      default = null;
    };
    authKeyFile = lib.mkOption {
      type = types.nullOr types.path;
      default = null;
    };
    settings = lib.mkOption {
      type = types.attrs;
      default = { };
      description = "YAML settings for moxapi, written to ~/.config/mox/moxapi/config.yaml";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile =
      {
        "mox/moxapi/config.yaml".text = lib.generators.toYAML { } cfg.settings;

        # Use the package's service file directly
        "systemd/user/moxapi.service".source = "${cfg.package}/share/systemd/user/moxapi.service";
      }
      // lib.optionalAttrs (cfg.authKey != null || cfg.authKeyFile != null) {
        # Override with drop-in file for environment variables (only if needed)
        "systemd/user/moxapi.service.d/override.conf".text = ''
          [Service]
          ${lib.optionalString (cfg.authKey != null) "Environment=AUTH_KEY=${cfg.authKey}"}
          ${lib.optionalString (
            cfg.authKeyFile != null
          ) "Environment=AUTH_KEY_FILE=${toString cfg.authKeyFile}"}
        '';
      };
  };
}
