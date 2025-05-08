{ pkgs, lib, config, inputs, ... }:

{


  # https://devenv.sh/packages/
  packages = [ pkgs.cargo-expand];

  # https://devenv.sh/languages/
  languages.rust = {
      enable = true;
      # https://devenv.sh/reference/options/#languagesrustchannel
      channel = "stable";

      components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
    };
  languages.cplusplus.enable = true;


  # https://devenv.sh/git-hooks/
  # git-hooks.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}
