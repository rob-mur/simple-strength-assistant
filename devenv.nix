{pkgs, ...}: {
  devcontainer.enable = true;
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

  env = {
    CHROMIUM_EXECUTABLE_PATH = "${pkgs.chromium}/bin/chromium";
    PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";
  };
  enterShell = ''
    export PATH="$PATH:/$DEVENV_PROFILE/bin/";
  '';

  scripts = {
    build.exec = "dx bundle --web --release --debug-symbols=false";
    format.exec = "cargo fmt";
    lint.exec = "./scripts/lint.sh";
  };

  processes = {
    serve.exec = "dx serve --port 8081";
    test-serve.exec = "dx serve --port 8080 --features test-mode";
  };

  git-hooks.hooks = {
    ci-checks = {
      enable = true;
      name = "Code quality checks (format, clippy, test, build)";
      entry = ''
        format
        lint
        build
      '';
      stages = ["pre-commit" "pre-push"];
    };
  };

  enterTest = ''
    cargo test
    npm run test:e2e
  '';
}
