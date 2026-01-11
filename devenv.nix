{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  packages = with pkgs; [git spec-kit claude-code gh];

  languages.rust.enable = true;
}
