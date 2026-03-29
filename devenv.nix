{ pkgs, ... }:
let
  backlog-md = pkgs.callPackage ./nix/backlog-md.nix { };
in
{
  devcontainer.enable = true;
  devcontainer.settings.updateContentCommand = "direnv allow ; devenv shell";
  packages = with pkgs; [
    git
    gh
    dioxus-cli
    wasm-bindgen-cli
    binaryen
    devcontainer
    claude-code
    gemini-cli-bin
    chromium
    bats
    backlog-md
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = [ "wasm32-unknown-unknown" ];
  };

  languages.javascript = {
    enable = true;
    npm = {
      enable = true;
      install.enable = true;
    };
  };

  dotenv.enable = true;
  env = {
    CHROMIUM_EXECUTABLE_PATH = "${pkgs.chromium}/bin/chromium";
    PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";
  };
  enterShell = ''
    export PATH="$PATH:/$DEVENV_PROFILE/bin/";
  '';

  scripts = {
    build.exec = "dx bundle --web --release --debug-symbols=false";
    format.exec = "cargo fmt && prettier --write .";
    lint.exec = "./scripts/lint.sh";
    ralph.exec = "./scripts/ralph.sh \"$@\"";
  };

  processes = {
    test-serve.exec = "dx serve --port 3000 --features test-mode";
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
    nixfmt.enable = true;
    prettier.enable = true;
    commitlint = {
      enable = true;
      name = "Validate commit message";
      entry = "npx commitlint --edit";
      stages = [ "commit-msg" ];
    };
    ci-checks = {
      enable = true;
      name = "Code quality checks (format, clippy, test, build)";
      entry = ''
        lint
        build
      '';
      stages = [
        "pre-commit"
        "pre-push"
      ];
    };
  };

  enterTest = ''
    set -e
    cargo test
    bats scripts/ralph.bats
    npm run test:e2e
  '';
}
