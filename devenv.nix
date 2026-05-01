{pkgs, ...}: {
  devcontainer.enable = true;
  devcontainer.settings.updateContentCommand = "direnv allow ; devenv shell -- true";
  packages = with pkgs; [
    git
    gh
    dioxus-cli
    wasm-bindgen-cli
    binaryen
    devcontainer
    chromium
    sops
    age
    buildah
    skopeo
    freefont_ttf
    claude-code
    jq
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
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
    FONTCONFIG_FILE = "${pkgs.makeFontsConf { fontDirectories = [ pkgs.freefont_ttf ]; }}";
  };
  enterShell = ''
    export PATH="$PATH:/$DEVENV_PROFILE/bin/";
  '';

  scripts = {
    build.exec = "dx bundle --web --release --debug-symbols=false";
    format.exec = "cargo fmt && prettier --write .";
    lint.exec = "./scripts/lint.sh";
  };

  processes = {
    test-serve.exec = "dx serve --port 3000";
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
    nixfmt.enable = false;
    prettier.enable = true;
    commitlint = {
      enable = true;
      name = "Validate commit message";
      entry = "npx commitlint --edit";
      stages = ["commit-msg"];
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
    npm run test:e2e
  '';
}
