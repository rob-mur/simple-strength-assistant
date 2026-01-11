{pkgs, ...}: {
  packages = with pkgs; [git spec-kit claude-code gh trunk];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
  };
}
